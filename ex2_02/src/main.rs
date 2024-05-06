use good_lp::{clarabel, constraint, variable, variables, Expression, Solution, SolverModel};
use itertools::izip;
use polars::prelude::*;
use std::collections::HashMap;

fn usage() {
    eprintln!("Usage: cargo run <oils_csv_path> <products_csv_path> [<solution_csv_path>]");
}

#[derive(Clone)]
struct Oil {
    name: String,
    pn: f64,
    rvp: f64,
    qy_max: u32,
}

impl Oil {
    fn new(name: &str, pn: f64, rvp: f64, qy_max: u32) -> Self {
        Self {
            name: String::from(name),
            pn,
            rvp,
            qy_max,
        }
    }
}

#[derive(Clone)]
struct Product {
    name: String,
    pn_min: Option<i64>,
    rvp_max: Option<f64>,
    price: f64,
}

impl Product {
    fn new(name: &str, pn_min: Option<i64>, rvp_max: Option<f64>, price: f64) -> Self {
        Self {
            name: String::from(name),
            pn_min,
            rvp_max,
            price,
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 3 || args.len() > 4 {
        usage();
        std::process::exit(1);
    }
    let oils_csv = &args[1];
    let products_csv = &args[2];

    // オイルの種類と性質
    let oils = CsvReader::from_path(oils_csv).unwrap().finish().unwrap();
    println!("{}", oils);

    // プロダクトの種類と制約, 売値
    let products = CsvReader::from_path(products_csv)
        .unwrap()
        .finish()
        .unwrap();
    println!("{}", products);

    let oil_map = {
        let mut map = HashMap::new();
        let oil_iter = izip!(
            oils["name"].str().unwrap().iter(),
            oils["pn"].f64().unwrap().iter(),
            oils["rvp"].f64().unwrap().iter(),
            oils["qy_max"].i64().unwrap().iter(),
        );
        for (name, pn, rvp, qy_max) in oil_iter {
            let oil = Oil::new(
                &name.unwrap(),
                pn.unwrap(),
                rvp.unwrap(),
                qy_max.unwrap() as u32,
            );
            map.insert(oil.name.clone(), oil.clone());
        }
        map
    };

    let product_map = {
        let mut map = HashMap::new();
        let product_iter = izip!(
            products["name"].str().unwrap().iter(),
            products["pn_min"].iter(),
            products["rvp_max"].iter(),
            products["price"].f64().unwrap().iter(),
        );
        for (name, pn_min, rvp_max, price) in product_iter {
            let pn_min = if let AnyValue::Int64(pn_min_inner) = pn_min {
                Some(pn_min_inner)
            } else {
                None
            };
            let rvp_max = if let AnyValue::Float64(rvp_max_inner) = rvp_max {
                Some(rvp_max_inner)
            } else {
                None
            };
            let product = Product::new(&name.unwrap(), pn_min, rvp_max, price.unwrap());
            map.insert(product.name.clone(), product.clone());
        }
        map
    };

    // 変数定義
    let mut variables = variables!();
    let mut variable_map: HashMap<(&str, &str), variable::Variable> = HashMap::new();
    for (oil_name, oil) in &oil_map {
        let qy_max = oil.qy_max;
        for (product_name, _product) in &product_map {
            variable_map.insert(
                (oil_name, product_name),
                variables.add(variable().bounds(0..qy_max)),
            );
        }
    }

    // 目的関数
    let mut objective = Expression::from_other_affine(0.);
    for (&(_oil, prod), var) in &variable_map {
        let price = product_map.get(prod).unwrap().price;
        objective += price * *var;
    }

    let mut problem = variables.maximise(&objective).using(clarabel);

    // オイルの生産量上限(合計)
    for (oil_name, oil) in &oil_map {
        let qy_max = oil.qy_max;
        let sum_qy = variable_map
            .iter()
            .filter(|(&(onm, _pnm), &_var)| onm == oil_name)
            .map(|(&_, var)| var)
            .sum::<Expression>();
        problem = problem.with(constraint!(sum_qy <= qy_max));
    }

    // 各混合液の PN 下限
    for (product_name, product) in &product_map {
        if let Some(pn_min) = product.pn_min {
            let pn_lhs = variable_map
                .iter()
                .filter(|(&(_onm, pnm), &_var)| pnm == product_name)
                .map(|(&(onm, _pnm), var)| oil_map.get(onm).unwrap().pn * var.clone())
                .sum::<Expression>();
            let pn_rhs = variable_map
                .iter()
                .filter(|(&(_onm, pnm), &_var)| pnm == product_name)
                .map(|(&(_onm, _pnm), var)| pn_min as f64 * var.clone())
                .sum::<Expression>();
            problem = problem.with(constraint!(pn_lhs >= pn_rhs));
        }
    }

    // 各混合液の RVP 上限
    for (product_name, product) in &product_map {
        if let Some(rvp_max) = product.rvp_max {
            let rvp_lhs = variable_map
                .iter()
                .filter(|(&(_onm, pnm), &_var)| pnm == product_name)
                .map(|(&(onm, _pnm), var)| oil_map.get(onm).unwrap().rvp * var.clone())
                .sum::<Expression>();
            let rvp_rhs = variable_map
                .iter()
                .filter(|(&(_onm, pnm), &_var)| pnm == product_name)
                .map(|(&(_onm, _pnm), var)| rvp_max * var.clone())
                .sum::<Expression>();
            problem = problem.with(constraint!(rvp_lhs <= rvp_rhs));
        }
    }

    let solution = problem.solve().unwrap();
    let optimal_value = solution.eval(&objective);

    println!();
    println!("optimal_value: {:.2}", optimal_value);

    let mut solution_df = df!(
        "oil" => variable_map.iter().map(|(&(onm, _pnm), _var)| onm.to_string()).collect::<Vec<String>>(),
        "product" => variable_map.iter().map(|(&(_onm, pnm), _var)| pnm.to_string()).collect::<Vec<String>>(),
        // "var_idx" => (0..solver.solution.x.len() as u32).collect::<Vec<u32>>(),
        "qy(solution)" => variable_map.iter()
            .map(|(&(_onm, _pnm), &var)| (solution.value(var) * 100.).round() / 100.)
            .collect::<Vec<f64>>(),
    )
    .unwrap();
    println!("{}", solution_df);

    if args.len() == 4 {
        let solution_csv = &args[3];
        let mut file = std::fs::File::create(solution_csv).unwrap();
        CsvWriter::new(&mut file).finish(&mut solution_df).unwrap();
    }
}
