use std::io;

use super::lazy_node::LazyNode;
use crate::adapt_response::adapt_response;
use clvmr::allocator::Allocator;
use clvmr::chia_dialect::ChiaDialect;
use clvmr::chia_dialect::{NO_NEG_DIV, NO_UNKNOWN_OPS};
use clvmr::cost::Cost;
use clvmr::deserialize_tree::{deserialize_tree, CLVMTreeBoundary};
use clvmr::reduction::Response;
use clvmr::run_program::run_program;
use clvmr::serialize::node_from_bytes;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
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

fn tuple_for_parsed_triple(py: Python<'_>, p: &CLVMTreeBoundary) -> PyObject {
    let tuple = match p {
        CLVMTreeBoundary::Atom {
            start,
            end,
            atom_offset,
        } => PyTuple::new(py, [*start, *end, *atom_offset as u64]),
        CLVMTreeBoundary::Pair {
            start,
            end,
            right_index,
        } => PyTuple::new(py, [*start, *end, *right_index as u64]),
    };
    tuple.into_py(py)
}

#[pyfunction]
fn deserialize_as_tree(py: Python, blob: &[u8]) -> PyResult<Vec<PyObject>> {
    let mut cursor = io::Cursor::new(blob);
    let r = deserialize_tree(&mut cursor)?;
    let r = r.iter().map(|pt| tuple_for_parsed_triple(py, pt)).collect();
    Ok(r)
}

#[pymodule]
fn clvm_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_serialized_program, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_as_tree, m)?)?;
    m.add("NO_NEG_DIV", NO_NEG_DIV)?;
    m.add("NO_UNKNOWN_OPS", NO_UNKNOWN_OPS)?;
    m.add_class::<LazyNode>()?;

    Ok(())
}
