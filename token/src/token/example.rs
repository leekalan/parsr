// TOKENS

use crate::{
    span::Spanned,
    token::{
        Associativity, FromStackEntry, HasStateTransition, IsOrdering, IsResolvedToken,
        OrderingBehaviour, StackEntry, TokenType,
    },
};

#[allow(unused)]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum RawToken {
    Let, // `let`
    Value(i32),
    Ident(String), // `$ident`
    Symbol(RawSymbol),
}

#[allow(unused)]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum RawSymbol {
    Assign,    // `=`
    List,      // `[ ..`
    Comma,     // `,`
    Semicolon, // `;`
    ListEnd,   // `]`
}

// RESOLVED TOKENS

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Term(Term),
    Operator(Operator),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Term {
    Value(i32),
    Ident(String),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Operator {
    Let,
    Assign,
    List,
    Comma,
    Semicolon,
}

// ORDERINGS

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Orderings {
    ListEnd,
}

// TREES

#[allow(unused)]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TokenTree {
    // Expr(ExprTree),
    Semicolon,
}

// #[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// enum ExprTree {
//     Term(TermTree),
//     Operator(OperatorTree),
//     ListEnd,
// }

// #[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub enum TermTree {
//     Value,
//     Ident,
// }

// #[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// enum OperatorTree {
//     Let,
//     Assign,
//     List,
//     Comma,
// }

// IS RESOLVED TOKEN

impl IsResolvedToken for Token {
    #[inline]
    fn get_type(&self) -> TokenType {
        match self {
            Token::Term(term) => term.get_type(),
            Token::Operator(operator) => operator.get_type(),
        }
    }
}

impl IsResolvedToken for Term {
    #[inline(always)]
    fn get_type(&self) -> TokenType {
        TokenType::Value
    }
}

impl IsResolvedToken for Operator {
    #[inline]
    fn get_type(&self) -> TokenType {
        match self {
            Operator::Let => TokenType::Precedence {
                precedence: 2,
                associativity: Associativity::Right,
            },
            Operator::Assign => TokenType::Precedence {
                precedence: 1,
                associativity: Associativity::Right,
            },
            Operator::List => TokenType::Precedence {
                precedence: 3,
                associativity: Associativity::ClosedRight,
            },
            Operator::Comma => TokenType::Precedence {
                precedence: 2,
                associativity: Associativity::Left,
            },
            Operator::Semicolon => TokenType::Precedence {
                precedence: 0,
                associativity: Associativity::Left,
            },
        }
    }
}

// IS ORDERING

impl IsOrdering for Orderings {
    #[inline]
    fn behaviour(&self) -> OrderingBehaviour {
        match self {
            Orderings::ListEnd => OrderingBehaviour::ClosedLeft,
        }
    }
}

// FROM STACK ENTRY

impl FromStackEntry for TokenTree {
    type Token = Token;
    type Ordering = Orderings;

    fn from_entry(_token: &StackEntry<Self::Token, Self::Ordering>) -> Self {
        Self::default()
    }
}

// HAS STATE TRANSITION

impl Default for TokenTree {
    fn default() -> Self {
        Self::Semicolon
    }
}

impl HasStateTransition<RawToken> for TokenTree {
    type Token = Token;
    type Ordering = Orderings;
    type Error = ();

    fn transition(
        self,
        token: RawToken,
    ) -> Result<StackEntry<Self::Token, Self::Ordering>, Spanned<Self::Error>> {
        match token {
            RawToken::Let => Ok(StackEntry::Resolved(Spanned::default_span(
                Token::Operator(Operator::Let),
            ))),
            RawToken::Value(value) => Ok(StackEntry::Resolved(Spanned::default_span(Token::Term(
                Term::Value(value),
            )))),
            RawToken::Ident(ident) => Ok(StackEntry::Resolved(Spanned::default_span(Token::Term(
                Term::Ident(ident),
            )))),
            RawToken::Symbol(raw_symbol) => match raw_symbol {
                RawSymbol::Assign => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Assign),
                ))),
                RawSymbol::List => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::List),
                ))),
                RawSymbol::Comma => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Comma),
                ))),
                RawSymbol::Semicolon => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Semicolon),
                ))),
                RawSymbol::ListEnd => Ok(StackEntry::Ordering(Spanned::default_span(
                    Orderings::ListEnd,
                ))),
            },
        }
    }
}

// impl HasStateTransition<Token> for LetTree {
//     type TokenTree = TokenTree;
//     type TransitionError = ();

//     fn transition(self, token: &Token) -> Result<Self::TokenTree, Self::TransitionError> {
//         Ok(match self {
//             LetTree::Let => match token {
//                 Token::Ident(_) => TokenTree::Let(LetTree::Ident),
//                 _ => return Err(()),
//             },
//             LetTree::Ident => match token {
//                 Token::Ordering(Ordering::Semicolon) => TokenTree::Semicolon,
//                 Token::Operator(Operator::Assign) => {
//                     TokenTree::Expr(ExprTree::Operator(OperatorTree::Assign))
//                 }
//                 _ => return Err(()),
//             },
//         })
//     }
// }

// impl HasStateTransition<Token> for ExprTree {
//     type TokenTree = TokenTree;
//     type TransitionError = ();

//     fn transition(self, token: &Token) -> Result<Self::TokenTree, Self::TransitionError> {
//         todo!()
//     }
// }

// PRINT OUT STRUCTURE

#[allow(unused)]
fn display_processed_tokens(tokens: impl Iterator<Item = Token>) -> String {
    let mut stack = Vec::new();

    for token in tokens {
        match token {
            Token::Term(term) => match term {
                Term::Value(val) => stack.push(format!("{val}")),
                Term::Ident(val) => stack.push(val),
            },
            Token::Operator(operator) => match operator {
                Operator::Let => {
                    let s = format!("let {}", stack.pop().unwrap());
                    stack.push(s)
                }
                Operator::Assign => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();

                    let s = format!("{left} = {right}");
                    stack.push(s)
                }
                Operator::List => {
                    let s = format!("[{}]", stack.pop().unwrap());
                    stack.push(s)
                }
                Operator::Comma => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();

                    let s = format!("{left}, {right}");
                    stack.push(s)
                }
                Operator::Semicolon => {
                    let s = format!("{};", stack.pop().unwrap());
                    stack.push(s)
                }
            },
        }
    }

    stack.pop().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::token::CreateTokenProcessor;

    use super::*;

    #[test]
    fn simple_test() {
        // let my_var = 42;
        let tokens = [
            RawToken::Let,
            RawToken::Ident("my_var".to_string()),
            RawToken::Symbol(RawSymbol::Assign),
            RawToken::Value(42),
            RawToken::Symbol(RawSymbol::Semicolon),
        ];

        let processed = CreateTokenProcessor::<RawToken, TokenTree, ()>::new(tokens.into_iter())
            .map(|t| t.unwrap().inner)
            .collect::<Vec<_>>();

        assert_eq!(
            processed,
            [
                Token::Term(Term::Ident("my_var".to_string())),
                Token::Operator(Operator::Let),
                Token::Term(Term::Value(42)),
                Token::Operator(Operator::Assign),
                Token::Operator(Operator::Semicolon),
            ]
        );

        println!("{}", display_processed_tokens(processed.into_iter()));
    }

    #[test]
    fn complext_test() {
        // let my_var = [ 13, 24, 35 ];
        let tokens = [
            RawToken::Let,
            RawToken::Ident("my_var".to_string()),
            RawToken::Symbol(RawSymbol::Assign),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Value(13),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Value(24),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Value(35),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::Semicolon),
        ];

        let processed = CreateTokenProcessor::<RawToken, TokenTree, ()>::new(tokens.into_iter())
            .map(|t| t.unwrap().inner)
            .collect::<Vec<_>>();

        assert_eq!(
            processed,
            [
                Token::Term(Term::Ident("my_var".to_string())),
                Token::Operator(Operator::Let),
                Token::Term(Term::Value(13)),
                Token::Term(Term::Value(24)),
                Token::Operator(Operator::Comma),
                Token::Term(Term::Value(35)),
                Token::Operator(Operator::Comma),
                Token::Operator(Operator::List),
                Token::Operator(Operator::Assign),
                Token::Operator(Operator::Semicolon),
            ]
        );

        println!("{}", display_processed_tokens(processed.into_iter()));
    }

    #[test]
    fn nested_list() {
        // let my_var = [ [1, 2], [3], 4, [ [5], 6] ];
        let tokens = [
            RawToken::Let,
            RawToken::Ident("my_var".to_string()),
            RawToken::Symbol(RawSymbol::Assign),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Value(1),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Value(2),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Value(3),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Value(4),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Symbol(RawSymbol::List),
            RawToken::Value(5),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::Comma),
            RawToken::Value(6),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::ListEnd),
            RawToken::Symbol(RawSymbol::Semicolon),
        ];

        let processed = CreateTokenProcessor::<RawToken, TokenTree, ()>::new(tokens.into_iter())
            .map(|t| t.unwrap().inner)
            .collect::<Vec<_>>();

        assert_eq!(
            processed,
            [
                Token::Term(Term::Ident("my_var".to_string())),
                Token::Operator(Operator::Let),
                Token::Term(Term::Value(1)),
                Token::Term(Term::Value(2)),
                Token::Operator(Operator::Comma),
                Token::Operator(Operator::List),
                Token::Term(Term::Value(3)),
                Token::Operator(Operator::List),
                Token::Operator(Operator::Comma),
                Token::Term(Term::Value(4)),
                Token::Operator(Operator::Comma),
                Token::Term(Term::Value(5)),
                Token::Operator(Operator::List),
                Token::Term(Term::Value(6)),
                Token::Operator(Operator::Comma),
                Token::Operator(Operator::List),
                Token::Operator(Operator::Comma),
                Token::Operator(Operator::List),
                Token::Operator(Operator::Assign),
                Token::Operator(Operator::Semicolon),
            ]
        );

        println!("{}", display_processed_tokens(processed.into_iter()));
    }
}
