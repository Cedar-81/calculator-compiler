use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "calc.pest"]
struct CalcParser;

#[derive(Debug, Clone)]
enum Expr {
    Number(String),
    Operator(Operator),
    None,
    PriorityArr(Vec<(Expr, i32)>),
    BinOp(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn main() {
    parser("(3 + 5 * 10 * 20 * 2 * 40 * 3 / 9 + 2) / 8 * (3 + 5 * 10 * 20)")
}

fn parser(calc: &str) {
    let block = CalcParser::parse(Rule::block, calc)
        .unwrap()
        .next()
        .unwrap()
        .into_inner();
    let val = String::new();
    let mut priority_arr: Vec<(Expr, i32)> = Vec::new();

    for pair in block {
        let result = build_ast(pair.clone(), &val);
        //println!("pair: {}\n result: {:?}", pair.as_str(), result);

        match result {
            Expr::PriorityArr(_) => {
                let value = build_bin_op(result); //consturct BinOp for brackets
                                                  //println!("bin_op: {:?}", value);
                priority_arr.push((value.unwrap(), 0))
            }
            Expr::Operator(Operator::Add) => priority_arr.push((result, 1)),
            Expr::Operator(Operator::Subtract) => priority_arr.push((result, 2)),
            Expr::Operator(Operator::Multiply) => priority_arr.push((result, 3)),
            Expr::Operator(Operator::Divide) => priority_arr.push((result, 4)),
            Expr::Number(_) => priority_arr.push((result, 0)),

            _ => (),
        }
    }

    let fin_bin_op = build_bin_op(Expr::PriorityArr(priority_arr.clone()));
    let result = calculate(fin_bin_op.clone().unwrap(), 0_f32);

    println!(
        "PriorityArr Data: {:?}\n\nFinalBinOp: {:?}\n\nCalcResult: {:?}",
        priority_arr,
        fin_bin_op, //construct BinOp for all
        result
    );
}

fn calculate(exp: Expr, sum_val: f32) -> Option<f32> {
    //println!("calledhere, {:?}", exp);
    let mut mut_sum = sum_val.clone();

    //println!("heres the mut sum: {}", mut_sum);

    match exp {
        Expr::BinOp(left, center, right) => {
            //println!("in here, {:?}", left);
            let l_val = calculate(*left, mut_sum);
            let r_val = calculate(*right, mut_sum);

            //println!("iniside here; {:?} {:?}", l_val, r_val);

            mut_sum += match (l_val, r_val) {
                (Some(val1), Some(val2)) => combine(val1, center, val2),

                (None, Some(val2)) => combine(mut_sum, center, val2),

                (None, None) => mut_sum,
                _ => mut_sum,
            };

            fn combine(val1: f32, center: Operator, val2: f32) -> f32 {
                match center {
                    Operator::Add => val1 + val2,
                    Operator::Subtract => val1 - val2,
                    Operator::Divide => val1 / val2,
                    Operator::Multiply => val1 * val2,
                    _ => 0_f32,
                }
            }

            //println!("finalstr: {}", sum_val);
            Some(mut_sum)
        }
        Expr::Number(num) => Some(num.trim().parse::<f32>().unwrap()),

        _ => None,
    }
}

fn build_bin_op(exp: Expr) -> Option<Expr> {
    let mut max_pr = 0;
    let mut max_idx = 0;

    let expr: Option<Expr> = match exp {
        Expr::PriorityArr(tuple_vec) => {
            for (index, &(_, priority)) in tuple_vec.iter().enumerate() {
                //println!("priority: {}, {}, {}", max_pr, max_idx, index);
                if priority > max_pr {
                    max_pr = priority;
                    max_idx = index;
                }
            }

            //println!("inhere, {} {:?}\n", max_idx, tuple_vec);

            let left_idx = max_idx - 1;
            let right_idx = max_idx + 1;

            let left = Box::new(tuple_vec[left_idx as usize].0.clone());
            let right = Box::new(tuple_vec[right_idx].0.clone());
            let center = tuple_vec[max_idx].0.clone();

            ///get modified arr, removing items that
            ///will be consturcted into BinOp
            let mut tv_clone = tuple_vec.clone();
            tv_clone.drain(left_idx as usize..=right_idx);

            ///extract Operator from enum Expr::Operator
            if let Expr::Operator(center_op) = center {
                let bin_op = Expr::BinOp(left, center_op, right);
                //println!("bin_opin: {:?}", bin_op);

                ///if all the items in tuple_vec have been consumed
                ///return constucted BinOp
                if tv_clone.len() == 0 {
                    return Some(bin_op);
                } else {
                    //tv_clone.append(&mut vec![(bin_op, 0)]);
                    /// add constructed BinOp into its original poistion
                    /// and call build_bin_op recursively
                    tv_clone.splice(left_idx..left_idx, vec![(bin_op, 0)]);
                    return build_bin_op(Expr::PriorityArr(tv_clone));
                }
            } else {
                return None;
            };
        }
        _ => None,
    };

    expr
}

fn build_ast(pair: Pair<Rule>, value: &str) -> Expr {
    let mut temp_val = String::new();
    temp_val = temp_val + value;
    //println!("insidepair: {} {:?}", pair, pair.as_rule());

    let match_val: Expr = match pair.as_rule() {
        Rule::expr => {
            temp_val = temp_val + "EXPR\n";
            let mut priority_arr: Vec<(Expr, i32)> = Vec::new();
            let mut counter = 0;
            //println!("tempvalexpr: {}", temp_val);

            for n_pair in pair.clone().into_inner() {
                //println!("n_pair: {}", n_pair);
                let n: Expr = build_ast(n_pair, format!("\t{}", value).as_str());
                counter += 1;

                match n {
                    Expr::Operator(Operator::Add) => priority_arr.push((n, 1)),
                    Expr::Operator(Operator::Subtract) => priority_arr.push((n, 2)),
                    Expr::Operator(Operator::Multiply) => priority_arr.push((n, 3)),
                    Expr::Operator(Operator::Divide) => priority_arr.push((n, 4)),
                    Expr::Number(_) => priority_arr.push((n, 0)),
                    _ => (),
                }
            }

            Expr::PriorityArr(priority_arr)
        }

        Rule::opr => build_ast(pair.into_inner().next().unwrap(), value),

        Rule::add => Expr::Operator(Operator::Add),

        Rule::subtract => Expr::Operator(Operator::Subtract),

        Rule::divide => Expr::Operator(Operator::Divide),

        Rule::multiply => Expr::Operator(Operator::Multiply),

        Rule::num => Expr::Number(pair.as_str().to_string()),

        _ => Expr::None,
    };

    //println!("data: {}", temp_val);
    match_val
}
