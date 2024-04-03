use clarabel_wrapper::*;
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

#[derive(Clone, Debug)]
struct Variable {
    idx: usize,
    oil: String,
    product: String,
}

impl Variable {
    fn new(idx: usize, oil: &str, product: &str) -> Self {
        Self {
            idx,
            oil: String::from(oil),
            product: String::from(product),
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

    let mut oil_map = HashMap::new();
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
        oil_map.insert(oil.name.clone(), oil.clone());
    }

    let mut product_map = HashMap::new();
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
        product_map.insert(product.name.clone(), product.clone());
    }

    let num_vars = oil_map.len() * product_map.len();
    let mut variables = Vec::with_capacity(num_vars);
    let mut idx = 0;
    for (_, oil) in &oil_map {
        for (_, product) in &product_map {
            variables.push(Variable::new(idx, &oil.name, &product.name));
            idx += 1;
        }
    }

    let mut model = Model::new();
    model.add_variables(num_vars);

    // 各変数は 0 以上
    for var in &variables {
        let mut coef = vec![0.; num_vars];
        coef[var.idx] = 1.;
        model.add_constraint_geq(&coef, 0.);
        println!("{:?} * x >= {}", &coef, 0.);
    }

    // 各オイルの生産量上限
    for (_, oil) in &oil_map {
        let mut coef = vec![0.; num_vars];
        for var in &variables {
            if var.oil == oil.name {
                coef[var.idx] = 1.;
            }
        }

        let qy_max = oil.qy_max as f64;

        model.add_constraint_leq(&coef, qy_max);
        println!("{:?} * x <= {}: {} 生産量上限", &coef, qy_max, oil.name);
    }

    // 各混合液の PN 下限
    for (_, product) in &product_map {
        if let Some(pn_min) = product.pn_min {
            let mut coef = vec![0.; num_vars];
            for var in &variables {
                if var.product == product.name {
                    let pn = oil_map.get(&var.oil).unwrap().pn;
                    coef[var.idx] = pn - (pn_min as f64);
                }
            }
            model.add_constraint_geq(&coef, 0.);
            println!("{:?} * x >= {}: {} PN 下限", &coef, pn_min, product.name);
        }
    }

    // 各混合液の RVP 上限
    for (_, product) in &product_map {
        if let Some(rvp_max) = product.rvp_max {
            let mut coef = vec![0.; num_vars];
            for var in &variables {
                if var.product == product.name {
                    let rvp = oil_map.get(&var.oil).unwrap().rvp;
                    coef[var.idx] = rvp - (rvp_max as f64);
                }
            }
            model.add_constraint_leq(&coef, 0.);
            println!("{:?} * x <= {}: {} RVP 上限", &coef, rvp_max, product.name);
        }
    }

    // 目的関数: maximize 売上
    let mut obj_coef = vec![0.; num_vars];
    for var in &variables {
        let price = product_map.get(&var.product).unwrap().price;
        obj_coef[var.idx] = price;
    }
    model.maximize(None, Some(&obj_coef));
    println!("objective: maximize {obj_coef:?} * x");

    let solver = model.solve();

    println!("Solution (x)    = {:?}", solver.solution.x);
    println!("Multipliers (z) = {:?}", solver.solution.z);
    println!("Slacks (s)      = {:?}", solver.solution.s);

    println!();
    println!(
        "optimal_value: {}",
        (-solver.solution.obj_val * 10000.).round() / 10000.
    );

    let mut solution_df = df!(
        "oil" => variables.iter().map(|v| v.oil.clone()).collect::<Vec<String>>(),
        "product" => variables.iter().map(|v| v.product.clone()).collect::<Vec<String>>(),
        "var_idx" => (0..solver.solution.x.len() as u32).collect::<Vec<u32>>(),
        "qy(solution)" => solver.solution.x.iter()
            .map(|x| (x * 10000.).round() / 10000.)
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
