use super::lazy_node::LazyNode;
use crate::adapt_response::adapt_response;
use clvmr::allocator::Allocator;
use clvmr::chia_dialect::ChiaDialect;
use clvmr::chia_dialect::{NO_NEG_DIV, NO_UNKNOWN_OPS};
use clvmr::cost::Cost;
use clvmr::reduction::Response;
use clvmr::run_program::run_program;
use clvmr::serialize::node_from_bytes;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
pub fn run_serialized_program(
    py: Python,
    program: &[u8],
    args: &[u8],
    max_cost: Cost,
    flags: u32,
) -> PyResult<(PyObject, LazyNode)> {
    let mut allocator = Allocator::new();

    let r: Response = (|| -> PyResult<Response> {
        let program = node_from_bytes(&mut allocator, program)?;
        let args = node_from_bytes(&mut allocator, args)?;
        let dialect = ChiaDialect::new(flags);

        Ok(py
            .allow_threads(|| run_program(&mut allocator, &dialect, program, args, max_cost, None)))
    })()?;
    adapt_response(py, allocator, r)
}

#[pymodule]
fn clvm_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_serialized_program, m)?)?;
    m.add("NO_NEG_DIV", NO_NEG_DIV)?;
    m.add("NO_UNKNOWN_OPS", NO_UNKNOWN_OPS)?;
    m.add_class::<LazyNode>()?;

    Ok(())
}
