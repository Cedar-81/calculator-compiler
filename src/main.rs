use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::io::{self, Write};

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
    Sqrt(String),
    Pow(String, String),
    Root(String, String),
    Cos(String),
    Sine(String),
    Tan(String),
}

fn main() {
    // Create a mutable String to store the user input
    let mut input = String::new();

    // Print a prompt to the user
    print!("Enter calculation: ");

    // Flush the output to ensure the prompt is displayed
    io::stdout().flush().expect("Failed to flush stdout");

    // Read a line from standard input and store it in the 'input' variable
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Print the user input
    println!("\nCalculating: {}", input);
    parser(input.as_str().trim());
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
        // parse each pair into its supposed type
        let result = build_ast(pair.clone(), &val);
        //println!("pair: {}\n result: {:?}", pair.as_str(), result);
        //println!("result: {:?}", result);

        // build a priority arr for each all aprsed pair
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
            Expr::Operator(Operator::Pow(_, _)) => priority_arr.push((result, 0)),
            Expr::Operator(Operator::Sqrt(_)) => priority_arr.push((result, 0)),
            Expr::Operator(Operator::Root(_, _)) => priority_arr.push((result, 0)),
            Expr::Operator(Operator::Sine(_)) => priority_arr.push((result, 0)),
            Expr::Operator(Operator::Cos(_)) => priority_arr.push((result, 0)),
            Expr::Operator(Operator::Tan(_)) => priority_arr.push((result, 0)),
            Expr::Number(_) => priority_arr.push((result, 0)),

            _ => (),
        }
    }

    //println!("parr: {:?}", priority_arr);

    if priority_arr.len() > 1 {
        let fin_bin_op = build_bin_op(Expr::PriorityArr(priority_arr.clone()));
        let result = calculate(fin_bin_op.clone().unwrap(), 0_f32);

        println!("\nCalculation Result: {:?}", result);
    } else {
        let result = calculate(priority_arr.clone()[0].0.clone(), 0_f32);
        println!("\nCalculation Result: {:?}", result);
    }
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
        Expr::Operator(Operator::Pow(num1, num2)) => Some(
            num1.parse::<f32>()
                .unwrap()
                .powf(num2.parse::<f32>().unwrap()),
        ),
        Expr::Operator(Operator::Sqrt(num1)) => Some(num1.parse::<f32>().unwrap().sqrt()),
        Expr::Operator(Operator::Root(num1, num2)) => Some(
            num1.parse::<f32>()
                .unwrap()
                .powf(1.0 / num2.parse::<f32>().unwrap()),
        ),
        Expr::Operator(Operator::Cos(num1)) => Some(num1.parse::<f32>().unwrap().cos()),
        Expr::Operator(Operator::Sine(num1)) => Some(num1.parse::<f32>().unwrap().sin()),
        Expr::Operator(Operator::Tan(num1)) => {
            let angle_degrees = num1.parse::<f32>().unwrap();
            // Check if the angle is exactly 90 degrees
            if (angle_degrees % 180.0) == 90.0 {
                //println!("The tangent of 90 degrees is undefined.");
                None
            } else {
                let angle_radians = angle_degrees.to_radians();
                let tan_result = angle_radians.tan();
                Some(tan_result)
            }
        }
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

            let mut left_idx;

            //for cases where calc begins with a -
            if max_idx as isize - 1 < 0 {
                left_idx = 0;
            } else {
                left_idx = max_idx - 1;
            }

            let right_idx = max_idx + 1;

            let left = Box::new(tuple_vec[left_idx as usize].0.clone());
            let right = Box::new(tuple_vec[right_idx].0.clone());
            let center = tuple_vec[max_idx].0.clone();

            //get modified arr, removing items that
            //will be consturcted into BinOp
            let mut tv_clone = tuple_vec.clone();
            tv_clone.drain(left_idx as usize..=right_idx);

            //extract Operator from enum Expr::Operator
            if let Expr::Operator(center_op) = center {
                let bin_op = Expr::BinOp(left, center_op, right);
                //println!("bin_opin: {:?}", bin_op);

                //if all the items in tuple_vec have been consumed
                //return constucted BinOp
                if tv_clone.len() == 0 {
                    return Some(bin_op);
                } else {
                    //tv_clone.append(&mut vec![(bin_op, 0)]);
                    // add constructed BinOp into its original poistion
                    // and call build_bin_op recursively
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
    //let mut temp_val = String::new();
    //temp_val = temp_val + value;
    //println!("insidepair: {} {:?}", pair, pair.as_rule());

    let match_val: Expr = match pair.as_rule() {
        Rule::expr => {
            let mut priority_arr: Vec<(Expr, i32)> = Vec::new();

            for n_pair in pair.clone().into_inner() {
                //println!("n_pair: {}", n_pair);
                let n: Expr = build_ast(n_pair, format!("\t{}", value).as_str());

                match n {
                    Expr::Operator(Operator::Add) => priority_arr.push((n, 1)),
                    Expr::Operator(Operator::Subtract) => priority_arr.push((n, 2)),
                    Expr::Operator(Operator::Multiply) => priority_arr.push((n, 3)),
                    Expr::Operator(Operator::Divide) => priority_arr.push((n, 4)),
                    Expr::Operator(Operator::Pow(_, _)) => priority_arr.push((n, 0)),
                    Expr::Operator(Operator::Sqrt(_)) => priority_arr.push((n, 0)),
                    Expr::Operator(Operator::Root(_, _)) => priority_arr.push((n, 0)),
                    Expr::Operator(Operator::Sine(_)) => priority_arr.push((n, 0)),
                    Expr::Operator(Operator::Cos(_)) => priority_arr.push((n, 0)),
                    Expr::Operator(Operator::Tan(_)) => priority_arr.push((n, 0)),
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

        Rule::pow => Expr::Operator(Operator::Pow(
            pair.clone()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_string(),
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::sqrt => Expr::Operator(Operator::Sqrt(
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::root => Expr::Operator(Operator::Root(
            pair.clone()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_string(),
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::sin => Expr::Operator(Operator::Sine(
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::cos => Expr::Operator(Operator::Cos(
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::tan => Expr::Operator(Operator::Tan(
            pair.into_inner().next().unwrap().as_str().to_string(),
        )),

        Rule::num => Expr::Number(pair.as_str().to_string()),

        _ => Expr::None,
    };

    //println!("data: {}", temp_val);
    match_val
}
