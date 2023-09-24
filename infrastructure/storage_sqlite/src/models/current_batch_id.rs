use crate::schema::current_batch_id;
use diesel::{Identifiable, Insertable, Queryable};

#[derive(Debug, Queryable, Identifiable, Insertable)]
#[diesel(table_name = current_batch_id)]
pub struct CurrentBatchId {
    pub id: i32,
}
