use parser::{
    ast_resolver::{ASTResolver, VarContext},
    dependancy_graph::{DependancyGraph, TopologicalSort},
    CellParser,
};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::common_types::{Cell, ComputeError, Expression, Index, ParsedCell, Value};
mod parser;

#[derive(Debug, Default)]
pub struct SpreadSheet {
    pub cells: HashMap<Index, Cell>,
    dependencies: DependancyGraph,
}

impl VarContext for SpreadSheet {
    fn get_variable(&self, index: Index) -> Option<Result<Value, ComputeError>> {
        self.get_computed(index)
    }
}

impl SpreadSheet {
    /// Adds the dependency graph for a cell based on its parsed representation.
    fn add_dependencies(&mut self, index: Index, cell: &Cell) {
        if let Some(Ok(ParsedCell::Expr(Expression {
            ref dependencies, ..
        }))) = cell.parsed_representation
        {
            self.dependencies.add_node(index, dependencies);
        } else {
            self.dependencies.add_node(index, &vec![]);
        }
    }

    /// Updates the dependency graph for a cell based on its parsed representation.
    fn update_dependencies(&mut self, index: Index, cell: &Cell) {
        if let Some(Ok(ParsedCell::Expr(Expression {
            ref dependencies, ..
        }))) = cell.parsed_representation
        {
            self.dependencies.change_node(index, dependencies);
        } else {
            self.dependencies.change_node(index, &vec![]);
        }
    }

    /// Computes the value of a cell based on its parsed representation.
    fn compute_cell(&self, cell: &Cell) -> Option<Result<Value, ComputeError>> {
        match cell.parsed_representation {
            Some(Ok(ParsedCell::Expr(ref expr))) => Some(ASTResolver::resolve(&expr.ast, self)),
            Some(Ok(ParsedCell::Value(ref value))) => Some(Ok(value.clone())),
            Some(Err(ref e)) => Some(Err(ComputeError::ParseError(e.0.clone()))),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn from_file_path(input_path: PathBuf) -> Self {
        let mut buffer = String::new();
        let mut f = File::open(input_path).expect("Cannot open file");
        f.read_to_string(&mut buffer)
            .expect("Cannot read file to string");

        let mut spreadsheet = Self::default();

        for (y, line) in buffer.lines().enumerate() {
            for (x, cell) in line.split('|').enumerate() {
                let cell = cell.trim().to_string();
                if cell.is_empty() {
                    continue;
                }
                spreadsheet.add_cell_and_compute(Index { x, y }, cell);
            }
        }

        spreadsheet
    }

    pub fn compute_all(&mut self) {
        let TopologicalSort { sorted, cycles } = self.dependencies.topological_sort();
        for idx in sorted {
            let Some(cell) = self.cells.get(&idx) else {
                continue;
            };
            if !cell.needs_compute {
                continue;
            }
            let computed = self.compute_cell(cell);

            let cell = self.cells.get_mut(&idx).expect("should not fail");
            cell.computed_value = computed;
            cell.needs_compute = false
        }

        for idx in cycles {
            let cell = self.cells.get_mut(&idx).expect("should not fail");
            if !cell.needs_compute {
                continue;
            }
            cell.computed_value = Some(Err(ComputeError::Cycle));
        }
    }

    pub fn get_computed(&self, index: Index) -> Option<Result<Value, ComputeError>> {
        self.cells.get(&index)?.computed_value.clone()
    }
    
    pub fn get_error(&self, index: Index) -> Option<ComputeError> {
        match &self.cells.get(&index)?.computed_value {
            Some(val) => match val {
                Ok(_) => None,
                Err(e) => Some(e.clone()),
            },
            None => None,
        }
        
    }

    pub fn add_cell_and_compute(&mut self, index: Index, raw: String) {
        let mut cell = Cell::from_raw(raw);
        CellParser::parse_cell(&mut cell);

        self.add_dependencies(index, &cell);

        cell.computed_value = self.compute_cell(&cell);
        cell.needs_compute = false;
        self.cells.insert(index, cell);

        let mut need_compute = false;
        for dep in self.dependencies.get_all_dependants(index) {
            if let Some(cell) = self.cells.get_mut(&dep) {
                cell.needs_compute = true;
                need_compute = true;
            }
        }
        if need_compute {
            self.compute_all();
        }
    }

    pub fn remove_cell(&mut self, index: Index) {
        let mut need_compute = false;
        for dep in self.dependencies.get_all_dependants(index) {
            if let Some(cell) = self.cells.get_mut(&dep) {
                cell.needs_compute = true;
                need_compute = true;
            }
        }

        self.dependencies.remove_node(index);
        self.cells.remove(&index);

        if need_compute {
            self.compute_all();
        }
    }

    pub fn mutate_cell(&mut self, index: Index, new_raw: String) {
        let mut new_cell = Cell::from_raw(new_raw);
        CellParser::parse_cell(&mut new_cell);
        new_cell.computed_value = self.compute_cell(&new_cell);
        new_cell.needs_compute = false;

        self.update_dependencies(index, &new_cell);

        let cell = self
            .cells
            .get_mut(&index)
            .expect("Expected valid index for mutate cell");
        *cell = new_cell;

        let mut need_compute = false;
        for dep in self.dependencies.get_all_dependants(index) {
            if let Some(cell) = self.cells.get_mut(&dep) {
                cell.needs_compute = true;
                need_compute = true;
            }
        }
        if need_compute {
            self.compute_all();
        }
    }

    pub fn get_raw(&self, index: &Index) -> Option<&str> {
        Some(&self.cells.get(index)?.raw_representation)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ref() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "=A5".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a1),
            Some(Err(ComputeError::UnfindableReference(_)))
        ));
    }

    #[test]
    fn test_circular() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };
        let a2 = Index { x: 0, y: 1 };
        spreadsheet.add_cell_and_compute(a1, "=A2".to_string());
        spreadsheet.add_cell_and_compute(a2, "=A1".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a1),
            Some(Err(ComputeError::Cycle))
        ));

        assert!(matches!(
            spreadsheet.get_computed(a2),
            Some(Err(ComputeError::Cycle))
        ));
    }

    #[test]
    fn test_mutate() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };
        let a2 = Index { x: 0, y: 1 };
        let a3 = Index { x: 0, y: 2 };

        spreadsheet.add_cell_and_compute(a3, "=A2 * 3".to_string());
        spreadsheet.add_cell_and_compute(a2, "=A1 * 2".to_string());
        spreadsheet.add_cell_and_compute(a1, "1".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a2),
            Some(Ok(Value::Number(2.0)))
        ));

        assert!(matches!(
            spreadsheet.get_computed(a3),
            Some(Ok(Value::Number(6.0)))
        ));

        spreadsheet.mutate_cell(a1, "7".to_string());
        assert!(matches!(
            spreadsheet.get_computed(a2),
            Some(Ok(Value::Number(14.0)))
        ));

        assert!(matches!(
            spreadsheet.get_computed(a3),
            Some(Ok(Value::Number(42.0)))
        ));
    }

    #[test]
    fn test_remove_cell() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };
        let a2 = Index { x: 0, y: 1 };

        spreadsheet.add_cell_and_compute(a1, "10".to_string());
        spreadsheet.add_cell_and_compute(a2, "=A1 * 2".to_string());

        spreadsheet.remove_cell(a1);

        assert!(matches!(
            spreadsheet.get_computed(a2),
            Some(Err(ComputeError::UnfindableReference(_)))
        ));
    }

    #[test]
    fn test_invalid_expression() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "=A1 +".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a1),
            Some(Err(ComputeError::ParseError(_)))
        ));
    }

    #[test]
    fn test_self_reference() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "=A1".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a1),
            Some(Err(ComputeError::Cycle))
        ));
    }

    #[test]
    fn test_indirect_circular_reference() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };
        let b1 = Index { x: 1, y: 0 };
        let c1 = Index { x: 2, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "=C1".to_string());
        spreadsheet.add_cell_and_compute(b1, "=A1 * 2".to_string());
        spreadsheet.add_cell_and_compute(c1, "=B1".to_string());

        assert!(matches!(
            spreadsheet.get_computed(a1),
            Some(Err(ComputeError::Cycle))
        ));
        assert!(matches!(
            spreadsheet.get_computed(b1),
            Some(Err(ComputeError::Cycle))
        ));
        assert!(matches!(
            spreadsheet.get_computed(c1),
            Some(Err(ComputeError::Cycle))
        ));
    }

    #[test]
    fn test_function_call() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };
        let b1 = Index { x: 1, y: 0 };
        let c1 = Index { x: 2, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "15".to_string());
        spreadsheet.add_cell_and_compute(b1, "23".to_string());
        spreadsheet.add_cell_and_compute(c1, "=sum(A1:B1)".to_string());
        let computed = spreadsheet.get_computed(c1);
        assert!(matches!(computed, Some(Ok(Value::Number(38.0)))));
    }

    #[test]
    fn test_string() {
        let mut spreadsheet = SpreadSheet::default();
        let a1 = Index { x: 0, y: 0 };

        spreadsheet.add_cell_and_compute(a1, "=\"hello\"".to_string());
        let computed = spreadsheet.get_computed(a1);
        let expected = String::from("hello");
        let result = computed.unwrap().unwrap();
        match result {
            Value::Text(t) => assert_eq!(t, expected),
            _ => panic!("Expected text"),
        }
    }
}
