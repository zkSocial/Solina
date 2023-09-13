use crate::schema::solvers;
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name=solvers)]
pub struct Solver {
    pub id: i32,
    pub address: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name=solvers)]
pub struct NewSolver {
    pub address: String,
}
