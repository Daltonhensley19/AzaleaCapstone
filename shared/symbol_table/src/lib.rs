#![allow(dead_code)]

use ariadne::{Label, Report, ReportKind, Source};
use thiserror::Error;

use std::{cell::Cell, path::Path};

use lexer::token::{Token, TokenKind};


// `SemanticError` represents a general failure of Azalea program's semantics
#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Failed to semantically analyze Azalea program.")]
    SemanticFail,
}

// `SemanticErrorReporter` helps with reporting pretty compiler errors for semantic stage
pub struct SemanticErrorReporter;

// Specific error report handlers
impl SemanticErrorReporter {
    pub fn dup_function_def<'a>(duplicated_func: &Token, path: &str, source: &str, offset: usize) {
        let note = format!(
            "funcion def `{0:?}` was found more than once.",
            duplicated_func.get_raw_content()
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(4)
            .with_message("Function Def. Name Repeated (semantic error)")
            .with_label(
                Label::new((path, offset..offset))
                    .with_message("Here")
                    .with_color(ariadne::Color::Red),
            )
            .with_note(note)
            .finish()
            .print((path, Source::from(source)))
            .unwrap();
    }

    pub fn dup_choice_def<'a>(duplicated_choice: &Token, path: &str, source: &str, offset: usize) {
        let note = format!(
            "choice def `{0:?}` was found more than once.",
            duplicated_choice.get_raw_content()
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(4)
            .with_message("Choice Def. Name Repeated (semantic error)")
            .with_label(
                Label::new((path, offset..offset))
                    .with_message("Here")
                    .with_color(ariadne::Color::Red),
            )
            .with_note(note)
            .finish()
            .print((path, Source::from(source)))
            .unwrap();
    }

    pub fn dup_structure_def<'a>(
        duplicated_structure: &Token,
        path: &str,
        source: &str,
        offset: usize,
    ) {
        let note = format!(
            "structure def `{0:?}` was found more than once.",
            duplicated_structure.get_raw_content()
        );
        Report::build(ReportKind::Error, path, offset)
            .with_code(4)
            .with_message("Structure Def. Name Repeated (semantic error)")
            .with_label(
                Label::new((path, offset..offset))
                    .with_message("Here")
                    .with_color(ariadne::Color::Red),
            )
            .with_note(note)
            .finish()
            .print((path, Source::from(source)))
            .unwrap();
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum SymbolKind {
    FuncCall,
    Global,
    PrimVar,
    StructVar,
    ListVar,
    ChoiceVar,
    ForLoopIndex,
    FuncParm,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Primitve {
    U32,
    F32,
    Bool,
    Text,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Prim(Primitve),
    Struct,
    Choice,
    Func,
    List(Primitve),
    Undetermined,
}

#[derive(Debug)]
pub struct SymbolNode {
    sym_name: Token,
    sym_ty: Type,
    sym_kind: Cell<SymbolKind>,
    sym_scope_depth: usize,
    sym_scope_breath: usize,
}

impl SymbolNode {
    pub fn new(
        sym_name: Token,
        sym_ty: Type,
        sym_scope_depth: usize,
        sym_scope_breath: usize,
    ) -> Self {
        SymbolNode {
            sym_name,
            sym_ty: sym_ty.clone(),
            sym_kind: Cell::new(Self::determine_sym_kind(sym_ty)),
            sym_scope_depth,
            sym_scope_breath,
        }
    }

    fn determine_sym_kind(sym_ty: Type) -> SymbolKind {
        match sym_ty {
            Type::Prim(_) => SymbolKind::PrimVar,
            Type::Struct => SymbolKind::StructVar,
            Type::Choice => SymbolKind::ChoiceVar,
            Type::Func => SymbolKind::FuncCall,
            Type::List(_) => SymbolKind::ListVar,
            Type::Undetermined => SymbolKind::PrimVar,
        }
    }

    pub fn refine_sym_kind_to(&self, sym_kind: SymbolKind) {
        self.sym_kind.set(sym_kind);
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    nodes: Vec<SymbolNode>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: SymbolNode) {
        self.nodes.push(node)
    }
}

pub fn check_for_dup_funcs_syms<'semantic>(
    st: &SymbolTable,
    path: &'semantic Path,
    cleaned_source: &'semantic str,
) -> Result<(), SemanticError> {
    for (idx, symbol_node) in st
        .nodes
        .iter()
        .filter(|sn| sn.sym_ty == Type::Func)
        .enumerate()
    {
        println!("{symbol_node:#?}");

        let curr_node = symbol_node;

        for (jdx, next_node) in st
            .nodes
            .iter()
            .filter(|sn| sn.sym_ty == Type::Func)
            .enumerate()
        {
            if curr_node.sym_name.get_raw_content() == next_node.sym_name.get_raw_content()
                && idx != jdx
            {
                SemanticErrorReporter::dup_function_def(
                    &curr_node.sym_name,
                    path.to_str().unwrap(),
                    cleaned_source,
                    curr_node.sym_name.get_file_index(),
                );

                return Err(SemanticError::SemanticFail);
            }
        }
    }

    Ok(())
}

pub fn check_for_dup_choice_syms<'semantic>(
    st: &SymbolTable,
    path: &'semantic Path,
    cleaned_source: &'semantic str,
) -> Result<(), SemanticError> {
    for (idx, symbol_node) in st
        .nodes
        .iter()
        .filter(|sn| sn.sym_ty == Type::Choice)
        .enumerate()
    {
        println!("{symbol_node:#?}");

        let curr_node = symbol_node;

        for (jdx, next_node) in st
            .nodes
            .iter()
            .filter(|sn| sn.sym_ty == Type::Choice)
            .enumerate()
        {
            if curr_node.sym_name.get_raw_content() == next_node.sym_name.get_raw_content()
                && idx != jdx
            {
                SemanticErrorReporter::dup_choice_def(
                    &curr_node.sym_name,
                    path.to_str().unwrap(),
                    cleaned_source,
                    curr_node.sym_name.get_file_index(),
                );

                return Err(SemanticError::SemanticFail);
            }
        }
    }

    Ok(())
}

pub fn check_for_dup_structs_syms<'semantic>(
    st: &SymbolTable,
    path: &'semantic Path,
    cleaned_source: &'semantic str,
) -> Result<(), SemanticError> {
    for (idx, symbol_node) in st
        .nodes
        .iter()
        .filter(|sn| sn.sym_ty == Type::Struct)
        .enumerate()
    {
        println!("{symbol_node:#?}");

        let curr_node = symbol_node;

        for (jdx, next_node) in st
            .nodes
            .iter()
            .filter(|sn| sn.sym_ty == Type::Struct)
            .enumerate()
        {
            if curr_node.sym_name.get_raw_content() == next_node.sym_name.get_raw_content()
                && idx != jdx
            {
                SemanticErrorReporter::dup_structure_def(
                    &curr_node.sym_name,
                    path.to_str().unwrap(),
                    cleaned_source,
                    curr_node.sym_name.get_file_index(),
                );

                return Err(SemanticError::SemanticFail);
            }
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
