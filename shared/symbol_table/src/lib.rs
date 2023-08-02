use lexer::token::Token;
use parser::Type;

enum SymbolKind {
    FuncDecl, 
    StructDecl, 
    ChoiceDecl, 
    Var
}

struct SymbolNode {
    sym_name:         Token,
    sym_ty:           Type,
    sym_kind:         SymbolKind,
    sym_scope_depth:  u16,
    sym_scope_breath: u16,
}

struct SymbolTable {
    nodes: Vec<SymbolNode>
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
