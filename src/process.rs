use syn::{Ident, Type, Error, Visibility, BareFnArg, parenthesized, braced};
use syn::parse::{Parse, ParseBuffer};
use syn::Token;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use std::collections::LinkedList;

pub struct Processes(pub Vec<Process>);

impl Parse for Processes {

    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let mut list = LinkedList::<Process>::new();
        while input.peek(Ident::peek_any) {
            list.push_front(input.parse()?);
        }

        let mut result = Vec::with_capacity(list.len());
        let mut err_list = LinkedList::new();
        for i in list {
            let maybe_compiled = i.compile();
            match maybe_compiled {
                Err(e) => err_list.push_front(e),
                Ok(v) => if err_list.is_empty() {
                    result.push(v)
                },
            }
        }

        if !err_list.is_empty() {
            let mut err = err_list.pop_back().unwrap();
            while !err_list.is_empty() {
                err.combine(err_list.pop_back().unwrap());
            }
            Err(err)
        } else {
            Ok(Processes(result))
        }
    }
}

pub struct Process {
    visibility: Visibility,
    name: Ident,
    input_ty: Type,
    args: Option<Vec<BareFnArg>>,
    result_ty: Type,
    body: compiler::FlowBody,
}

impl Parse for Process {

    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let visibility = input.parse()?;
        let name = input.parse()?;
        let args_paren;
        parenthesized!(args_paren in input);
        let input_ty = args_paren.parse()?;
        let args = if input.peek(Token![,]) {
            let mut vec: Vec<BareFnArg> = Vec::with_capacity(8);
            while input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                let arg: BareFnArg = input.parse()?;
                if arg.name.is_none() {
                    abort!(
                        arg.span(),
                        "Expected name for this argument"
                    );
                }
                vec.push(arg);
            }
            Some(vec)
        } else {
            None
        };
        input.parse::<Token![->]>()?;
        let result_ty = input.parse()?;
        let body;
        braced!(body in input);
        Ok(Process {
            visibility,
            name,
            input_ty,
            args,
            result_ty,
            body: compiler::compile(&body)?,
        })
    }
}

mod compiler {
    use super::Error;
    use std::collections::LinkedList;
    use proc_macro_error::proc_macro::{TokenStream};
    use syn::parse::{Parse, ParseBuffer};
    use syn::{Ident, TypeParen, ExprParen, Token};
    use syn::parse::discouraged::Speculative;
    use syn::ext::IdentExt;
    use syn::group::parse_parens;
    use std::iter::FromIterator;

    /// Procedure or Process which can be called to perform operations in the process.
    pub struct Call {
        name: String,

        /// None if parentheses are absent and Some if are present.
        /// The array represents each Procedure/Process call if any.
        /// Array will be empty if parentheses were left empty.
        args: Option<Vec<Call>>,
    }

    impl Into<Stmt> for Call {

        fn into(self) -> Stmt {
            Stmt::CallStmt(self)
        }
    }

    impl Parse for Call {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            let name = input.parse::<Ident>()?.to_string();
            if input.peek(syn::token::Paren) {
                let mut args = LinkedList::<Call>::new();
                let tokens = parse_parens(input)?.content;
                while !tokens.is_empty() {
                    let call = tokens.parse::<Call>()?;
                    args.push_back(call);
                    if tokens.is_empty() {
                        break;
                    }
                    let lookahead = tokens.lookahead1();
                    if !lookahead.peek(Token![,]) {
                        return Err(lookahead.error());
                    }
                }
                return Ok(Call {
                    name,
                    args: Some(Vec::from_iter(args.into_iter())),
                });
            }
            Ok(Call {
                name,
                args: None,
            })
        }
    }

    pub struct AsyncBlock {
        body: FlowBody,
    }

    impl Into<Stmt> for AsyncBlock {

        fn into(self) -> Stmt {
            Stmt::AsyncBlockStmt(self)
        }
    }

    impl Parse for AsyncBlock {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            let ident = input.call(Ident::parse_any)?;
            if ident.to_string() != "async" {
                abort!(
                    ident.span(),
                    "Not an `async` block"
                );
            }

            let body = input.parse::<FlowBody>()?;
            Ok(AsyncBlock { body })
        }
    }

    // Block that constructs synchronous flow inside of asynchronous block.
    pub struct SyncBlock {
        body: FlowBody,
    }

    impl Into<Stmt> for SyncBlock {

        fn into(self) -> Stmt {
            Stmt::SyncBlockStmt(self)
        }
    }

    impl Parse for SyncBlock {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            let ident = input.parse::<Ident>()?;
            if ident.to_string() != "sync" {
                abort!(
                    ident.span(),
                    "Not an `sync` block"
                );
            }
            let body = input.parse::<FlowBody>()?;
            Ok(SyncBlock { body })
        }
    }

    pub struct If {
        condition: ConditionExpr,
        body: FlowBody,
    }

    impl Parse for If {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            let ident = input.call(Ident::parse_any)?;
            if ident.to_string() != "if" {
                abort!(
                    ident.span(),
                    "Not an `if` statement"
                );
            }

            let condition = input.parse::<ConditionExpr>()?;
            let body = input.parse::<FlowBody>()?;
            Ok(If {
                condition,
                body,
            })
        }
    }

    impl Into<Stmt> for If {

        fn into(self) -> Stmt {
            Stmt::IfStmt(self)
        }
    }

    pub struct FlowBody {
        stmts: LinkedList<Stmt>,
    }

    pub struct Return {
        call: Call,
    }

    /// All types of statements possible.
    pub enum Stmt {
        CallStmt(Call),
        BoolProcedureCallStmt(BoolProcedureCall),
        AsyncBlockStmt(AsyncBlock),
        SyncBlockStmt(SyncBlock),
        IfStmt(If),
        FlowBodyStmt(FlowBody),
        ReturnStmt(Return),
    }

    /// Expression types for if statement.
    pub enum ConditionExpr {
        Or {
            left: Box<ConditionExpr>,
            right: Box<ConditionExpr>,
        },
        And {
            left: Box<ConditionExpr>,
            right: Box<ConditionExpr>,
        },
        Eq {
            left: Box<ConditionExpr>,
            right: Box<ConditionExpr>,
        },
        Neq {
            left: Box<ConditionExpr>,
            right: Box<ConditionExpr>,
        },
        Not {
            expr: Box<ConditionExpr>,
        },
        CallExpr(Call),
    }

    /// Operations of boolean expression that have two operands.
    /// Used to help parsing those expressions.
    enum Op {
        Or,
        And,
        Eq,
        Neq,
    }

    impl Parse for ConditionExpr {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            use ConditionExpr::*;

            if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                let expr = input.parse::<ConditionExpr>()?;
                return Ok(ConditionExpr::Not { expr: Box::new(expr) });
            }

            let parse_side = || -> Result<ConditionExpr, Error> {
                if input.peek(syn::token::Paren) {
                    let parens = parse_parens(input)?;
                    parens.content.parse::<ConditionExpr>()
                } else {
                    let call = input.parse::<Call>()?;
                    Ok(ConditionExpr::CallExpr(call))
                }
            };

            let left = Box::new(parse_side()?);

            let op;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![||]) {
                input.parse::<Token![||]>();
                op = Op::Or;
            } else if lookahead.peek(Token![&&]) {
                input.parse::<Token![&&]>();
                op = Op::And;
            } else if lookahead.peek(Token![==]) {
                input.parse::<Token![==]>();
                op = Op::Eq;
            } else if lookahead.peek(Token![!=]) {
                input.parse::<Token![!=]>();
                op = Op::Neq;
            } else {
                return Err(lookahead.error());
            }

            let right = Box::new(parse_side()?);

            let expr = match op {
                Op::Or => Or {
                    left,
                    right,
                },
                Op::And => And {
                    left,
                    right,
                },
                Op::Eq => Eq {
                    left,
                    right,
                },
                Op::Neq => Neq {
                    left,
                    right,
                },
            };

            Ok(expr)
        }
    }

    impl Parse for FlowBody {

        fn parse(input: &ParseBuffer) -> Result<Self, Error> {
            let mut errors = LinkedList::new();
            let mut stmts = LinkedList::<Stmt>::new();

            // Add statement to the list if appropriate.
            let add_stmt = |stmt: Stmt| {
                if errors.is_empty() {
                    stmts.push_front(stmt);
                }
                // Otherwise, there is no need to add statements as they will be discarded.
                // At this point we just collect errors.
            };

            while !input.is_empty() {
                let fork = input.fork();
                let stmt: Result<Stmt, Error> =
                    match input.call(Ident::parse_any)?.to_string().as_str() {
                    "if" => fork.parse::<If>().map(|v| v.into()),
                    "async" => fork.parse::<AsyncBlock>().map(|v| v.into()),
                    "sync" => fork.parse::<SyncBlock>().map(|v| v.into()),
                    _ => fork.parse::<Call>().map(|v| v.into()),
                };
                input.advance_to(&fork);
                match stmt {
                    Ok(s) => add_stmt(s),
                    Err(e) => errors.push_back(e),
                }
            }

            if errors.is_empty() {
                Ok(FlowBody { stmts })
            } else {
                let mut err = errors.pop_front().unwrap();
                while !errors.is_empty() {
                    err.combine(errors.pop_front().unwrap());
                }
                Err(err)
            }
        }
    }

    impl FlowBody {

        pub fn expand(&self) -> TokenStream {
            // TODO
            let expanded = quote!();

            TokenStream::from(expanded)
        }
    }
}
