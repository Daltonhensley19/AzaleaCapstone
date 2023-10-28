

use symbol_table::SymbolTable;
use symbol_table::SemanticError;
use parser::ast::*;



pub fn check_for_missing_varbind<'semantic>(st: &SymbolTable, ast: &Program) -> Result<(), SemanticError> {

    println!("{ast:#?}");
    
    if ast.declarations.is_none() {
        return Ok(());
    }

    for decls in ast.declarations.as_ref().unwrap() {
    }

    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
