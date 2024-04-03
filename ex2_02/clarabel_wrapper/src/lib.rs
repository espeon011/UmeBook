use clarabel::algebra::*;
use clarabel::solver::*;

pub use clarabel::algebra::CscMatrix;
pub use clarabel::solver::DefaultSolver;
pub use clarabel::solver::SupportedConeT;

pub struct Model {
    variable_num: usize,
    constraint_num: usize,
    p: CscMatrix<f64>,
    q: Vec<f64>,
    a: CscMatrix<f64>,
    b: Vec<f64>,
    k: Vec<SupportedConeT<f64>>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            variable_num: 0,
            constraint_num: 0,
            p: CscMatrix::zeros((0, 0)),
            q: Vec::new(),
            a: CscMatrix::zeros((0, 0)),
            b: Vec::new(),
            k: Vec::new(),
        }
    }

    pub fn add_variables(&mut self, variable_num: usize) {
        self.variable_num += variable_num;

        let p00 = &self.p;
        let p01: CscMatrix<f64> = CscMatrix::zeros((p00.m, variable_num));
        let p1011: CscMatrix<f64> = CscMatrix::zeros((variable_num, p00.n + variable_num));
        self.p = CscMatrix::vcat(&CscMatrix::hcat(p00, &p01), &p1011);

        self.q.resize(self.q.len() + variable_num, 0.);

        let a01: CscMatrix<f64> = CscMatrix::zeros((self.a.m, self.a.n + variable_num));
        self.a = CscMatrix::hcat(&self.a, &a01);
    }

    pub fn add_constraint(&mut self, acol: &[f64], bval: f64, kval: SupportedConeT<f64>) {
        self.a = CscMatrix::vcat(&self.a, &CscMatrix::from([acol]));
        self.b.push(bval);
        self.k.push(kval);
        self.constraint_num += 1;
    }

    pub fn add_constraint_eq(&mut self, acol: &[f64], bval: f64) {
        self.add_constraint(acol, bval, ZeroConeT(1));
    }

    pub fn add_constraint_leq(&mut self, acol: &[f64], bval: f64) {
        self.add_constraint(acol, bval, NonnegativeConeT(1));
    }

    pub fn add_constraint_geq(&mut self, acol: &[f64], bval: f64) {
        let acol_minus = acol.iter().map(|x| -x).collect::<Vec<f64>>();
        self.add_constraint_leq(&acol_minus, -bval);
    }

    fn set_p(&mut self, p: &CscMatrix) {
        self.p = p.clone();
    }

    fn set_q(&mut self, q: &[f64]) {
        self.q = Vec::from(q);
    }

    fn set_objective(&mut self, p: &CscMatrix, q: &[f64]) {
        self.set_p(p);
        self.set_q(q);
    }

    pub fn minimize(&mut self, p: Option<&CscMatrix>, q: Option<&[f64]>) {
        let p_inner = if let Some(p_inner) = p {
            p_inner.clone()
        } else {
            CscMatrix::zeros((self.variable_num, self.variable_num))
        };

        let q_inner = if let Some(q_inner) = q {
            Vec::from(q_inner)
        } else {
            vec![0.; self.variable_num]
        };

        self.set_objective(&p_inner, &q_inner);
    }

    pub fn maximize(&mut self, p: Option<&CscMatrix>, q: Option<&[f64]>) {
        let mut p_minus = if let Some(p_inner) = p {
            p_inner.clone()
        } else {
            CscMatrix::zeros((self.variable_num, self.variable_num))
        };
        p_minus.negate();

        let q_minus = if let Some(q_inner) = q {
            Vec::from(q_inner)
                .into_iter()
                .map(|x| -x)
                .collect::<Vec<f64>>()
        } else {
            vec![0.; self.variable_num]
        };

        self.set_objective(&p_minus, &q_minus);
    }

    pub fn solve(&self) -> DefaultSolver {
        let settings = DefaultSettings::default();

        let mut solver = DefaultSolver::new(&self.p, &self.q, &self.a, &self.b, &self.k, settings);

        solver.solve();

        solver
    }
}
