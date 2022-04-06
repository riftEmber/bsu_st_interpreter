//! AST node definitions

use std::collections::HashMap;
use std::time::Duration;

use chrono::naive::{NaiveDate, NaiveTime};
use num::{
    checked_pow, rational::Ratio, BigRational, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub,
    FromPrimitive, Integer, Signed, ToPrimitive,
};

use crate::ast::AddExpression::Add;
use crate::ast::AndExpression::And;
use crate::ast::AssignmentStatement::Asgn;
use crate::ast::Comparison::CompEq;
use crate::ast::ComparisonOperator::*;
use crate::ast::EquExpression::Equ;
use crate::ast::Expression::Expr;
use crate::ast::PowerExpression::Power;
use crate::ast::Term::Term as TermInstance;
use crate::ast::UnaryExpression::Unary;
use crate::ast::VariableValue::*;
use crate::ast::VarsDec::DecList;
use crate::ast::XorExpression::Xor;
use crate::prog_handle::InterpreterResult;
use crate::prog_handle::ProgContext;

/// Trait containing functionality for executable AST nodes
pub trait AstNode {
    /// Execute this node in the given context
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>>;
}

#[derive(Debug, Clone, PartialEq)]
/// ST variable types, each holding a corresponding value of that type.
pub enum VariableValue {
    INT(i16),
    BOOL(bool),
    BYTE(u8),
    WORD(u16),
    UINT(u16),
    DWORD(u32),
    DINT(i32),
    REAL(f32),
    LREAL(f64),
    CHAR(u8),
    WCHAR(u16),
    STRING(String),
    TIME(Duration),
    LTIME(Duration),
    DATE(NaiveDate),
    TimeOfDay(NaiveTime),
}

// Start of expressions

#[derive(Debug, Clone)]
/// Root node of expression branch.
/// Note that order of operations is captured in a leaf first resolution
/// of the various child nodes.
pub enum Expression {
    Expr(XorExpression, Option<XorExpression>),
}

impl AstNode for Expression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Expr(left, right) = self;
        let right = match right {
            Some(expr) => Some(expr.execute(context)?.unwrap()),
            None => None,
        };
        InterpreterResult::Ok(Some(boolean_operation_result(
            left.execute(context)?.unwrap(),
            BoolOp::OR,
            right,
        )?))
    }
}

#[derive(Debug, Clone)]
pub enum XorExpression {
    Xor(AndExpression, Option<AndExpression>),
}

impl AstNode for XorExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Xor(left, right) = self;
        let right = match right {
            Some(expr) => Some(expr.execute(context)?.unwrap()),
            None => None,
        };
        InterpreterResult::Ok(Some(boolean_operation_result(
            left.execute(context)?.unwrap(),
            BoolOp::XOR,
            right,
        )?))
    }
}

#[derive(Debug, Clone)]
pub enum AndExpression {
    And(Comparison, Option<Comparison>),
}

impl AstNode for AndExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let And(left, right) = self;
        let right = match right {
            Some(expr) => Some(expr.execute(context)?.unwrap()),
            None => None,
        };
        InterpreterResult::Ok(Some(boolean_operation_result(
            left.execute(context)?.unwrap(),
            BoolOp::AND,
            right,
        )?))
    }
}

#[derive(Debug, Clone)]
pub enum Comparison {
    CompEq(EquExpression, Option<(bool, EquExpression)>),
}

impl AstNode for Comparison {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let CompEq(left, op_and_right) = self;
        let left = left.execute(context)?.unwrap();
        InterpreterResult::Ok(Some(if let Some((is_equals, right)) = op_and_right {
            let right = right.execute(context)?.unwrap();
            let result = if *is_equals {
                left == right
            } else {
                left != right
            };
            BOOL(result)
        } else {
            left
        }))
    }
}

#[derive(Debug, Clone)]
pub enum EquExpression {
    Equ(AddExpression, Option<(ComparisonOperator, AddExpression)>),
}

impl AstNode for EquExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Equ(left, op_and_right) = self;
        let left = left.execute(context)?.unwrap();
        InterpreterResult::Ok(Some(if let Some((op, right)) = op_and_right {
            let right = right.execute(context)?.unwrap();
            comparison_operation_result(left, op, right)?
        } else {
            left
        }))
    }
}

#[derive(Debug, Clone)]
pub enum AddExpression {
    Add(Term, Option<(AddOperator, Term)>),
}

impl AstNode for AddExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Add(left, op_and_right) = self;
        let left = left.execute(context)?.unwrap();
        InterpreterResult::Ok(Some(if let Some((op, right)) = op_and_right {
            let right = right.execute(context)?.unwrap();
            math_operation_result(left, MathOp::Add(op.clone()), right)?
        } else {
            left
        }))
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    Term(PowerExpression, Option<(MultiplyOperator, PowerExpression)>),
}

impl AstNode for Term {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let TermInstance(left, op_and_right) = self;
        let left = left.execute(context)?.unwrap();
        InterpreterResult::Ok(Some(if let Some((op, right)) = op_and_right {
            let right = right.execute(context)?.unwrap();
            math_operation_result(left, MathOp::Multiply(op.clone()), right)?
        } else {
            left
        }))
    }
}

#[derive(Debug, Clone)]
pub enum PowerExpression {
    Power(UnaryExpression, Option<UnaryExpression>),
}

impl AstNode for PowerExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Power(left, right) = self;
        let left = left.execute(context)?.unwrap();
        let result = match right {
            Some(right) => {
                let right = right.execute(context)?.unwrap();
                math_operation_result(left, MathOp::Exponentiate, right)?
            }
            None => left,
        };

        InterpreterResult::Ok(Some(result))
    }
}

#[derive(Debug, Clone)]
pub enum UnaryExpression {
    Unary(PrimaryExpression, Option<UnaryOperator>), //Note order flipped for consistency
}

impl AstNode for UnaryExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Unary(expression, operator) = self;
        let expression_value = expression.execute(context)?.unwrap();
        let result = match operator {
            Some(op) => match op {
                UnaryOperator::NEGATIVE => match expression_value {
                    INT(x) => INT(-x),
                    DINT(x) => DINT(-x),
                    REAL(x) => REAL(-x),
                    LREAL(x) => LREAL(-x),
                    _ => {
                        return InterpreterResult::Err(String::from(
                            "Attempted to negate a type that cannot be negated",
                        ));
                    }
                },
                UnaryOperator::NOT => match expression_value {
                    BOOL(x) => BOOL(!x),
                    _ => {
                        return InterpreterResult::Err(String::from(
                            "Attempted to invert a non-boolean value",
                        ));
                    }
                },
            },
            None => expression_value,
        };

        InterpreterResult::Ok(Some(result))
    }
}

#[derive(Debug, Clone)]
pub enum PrimaryExpression {
    Const(VariableValue),
    VarName(Box<String>),
    Expr(Box<Expression>),
}

impl AstNode for PrimaryExpression {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        InterpreterResult::Ok(match self {
            PrimaryExpression::Const(value) => Some(value.clone()),
            PrimaryExpression::VarName(var_name) => Some(
                context
                    .get_var(*(*var_name).clone())
                    .ok_or(format!("Could not find referenced variable '{}'", var_name))?
                    .var_value
                    .clone(),
            ),
            PrimaryExpression::Expr(expression) => Some(expression.execute(context)?.unwrap()),
        })
    }
}

// Start of expression operators
#[derive(Debug, Clone)]
/// Different comparison operators for resolving boolean expressions
pub enum ComparisonOperator {
    LessThan,
    GreaterThan,
    LessEqualThan,
    GreaterEqualThan,
}

#[derive(Debug, Clone)]
/// Operators at the precedence of addition
pub enum AddOperator {
    ADD,
    SUBTRACT,
}

#[derive(Debug, Clone)]
/// Operators at the precedence of multiplication
pub enum MultiplyOperator {
    MULTIPLY,
    DIVIDE,
    MODULO,
}

#[derive(Debug, Clone)]
/// Operators at the precedence of unary operations
pub enum UnaryOperator {
    NEGATIVE,
    NOT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Different 'kinds' of ST variables, such as input, output, etc.
pub enum VariableKind {
    NORMAL,
    INPUT,
    OUTPUT,
    InOut,
    EXTERNAL,
    GLOBAL,
}
// End of expressions and operators

#[derive(Debug, Clone)]
/// A list of variable declarations of a certain variable kind
pub enum VarsDec {
    DecList(VariableKind, Box<HashMap<Box<String>, VariableValue>>),
}

impl AstNode for VarsDec {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let DecList(kind, decs) = self;
        for var_dec in decs.iter() {
            let var_name = (**var_dec.0).clone();
            let var_type = &*var_dec.1;

            context.add_var(var_name, kind.clone(), var_type.clone())?;
        }

        InterpreterResult::Ok(None)
    }
}

/// Representation of a generic statement
#[derive(Debug, Clone)]
pub enum Statement {
    Asgn(AssignmentStatement),
    // Assignment
    Select(SelectionStatement),
    // If statement
    Iter(IterationStatement), // While loop
}

/// A selection statement
#[derive(Debug, Clone)]
pub enum SelectionStatement {
    If(IfStatement),
}

/// Representation of an if statement
///
/// First value is a list of expressions and statement list touples.
/// This represents the conditonal and it's codeblock for each level of the if else ladder.
/// e.g the first touple in the list would be the "if", the second the "if-else", and the third the second "if-else"
///
/// The second value is an optional list of statements, this represents the optional final else of the if-else ladder.
#[derive(Debug, Clone)]
pub enum IfStatement {
    If(Vec<(Expression, Vec<Statement>)>, Option<Vec<Statement>>),
}

/// A iteration statement
#[derive(Debug, Clone)]
pub enum IterationStatement {
    While(WhileStatement),
}

/// Representation of a while loop
///
/// First value is the expression evaluated each loop, second is list of statements inside the while loop
#[derive(Debug, Clone)]
pub enum WhileStatement {
    While(Expression, Vec<Statement>),
}

/// A single assignment statement.
#[derive(Debug, Clone)]
pub enum AssignmentStatement {
    Asgn(Box<String>, Expression),
}

impl AstNode for AssignmentStatement {
    fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
        let Asgn(var_name, new_value) = self;
        let var_name = *var_name.clone();

        let new_value = new_value.execute(context)?.unwrap();
        context.update_var(&var_name, new_value)?;

        InterpreterResult::Ok(None)
    }
}

/// AST root node containing an entire ST program.
/// First arg is name, Second arg is varlist, third is statement list
#[derive(Debug, Clone)]
pub enum Program {
    Prog(Box<String>, Option<Vec<Box<VarsDec>>>, Vec<Statement>),
}

// TODO: Fix, broken due to changes introduced in subset 6 and 7
// impl AstNode for Program {
//     fn execute(&self, context: &mut ProgContext) -> InterpreterResult<Option<VariableValue>> {
//         let Prog(_, all_dec_lists, statements) = self;
//
//         // process variable declarations lists if present
//         if let Some(program_dec_lists) = all_dec_lists {
//             for dec_list in program_dec_lists {
//                 dec_list.execute(context);
//             }
//         }
//
//         // execute all statements (assignments) sequentially
//         for statement in statements {
//             statement.execute(context);
//         }
//
//         // this is the top level, so no evaluation value
//         None
//     }
// }

pub enum MathOp {
    Multiply(MultiplyOperator),
    Add(AddOperator),
    Exponentiate,
}

pub enum BoolOp {
    XOR,
    OR,
    AND,
}

fn boolean_operation_result(
    left: VariableValue,
    op: BoolOp,
    right: Option<VariableValue>,
) -> InterpreterResult<VariableValue> {
    InterpreterResult::Ok(if let Some(right) = right {
        let (left, right) = match (left, right) {
            (BOOL(left), BOOL(right)) => (left, right),
            (_, _) => {
                return InterpreterResult::Err(String::from(
                    "Attempted boolean operation with non-boolean values",
                ));
            }
        };
        BOOL(match op {
            BoolOp::XOR => left ^ right,
            BoolOp::OR => left | right,
            BoolOp::AND => left & right,
        })
    } else {
        left
    })
}

fn get_num_from_variable(variable: VariableValue) -> InterpreterResult<BigRational> {
    InterpreterResult::Ok(
        match variable {
            INT(x) => Ratio::from_i16(x),
            BYTE(x) => Ratio::from_u8(x),
            WORD(x) => Ratio::from_u16(x),
            UINT(x) => Ratio::from_u16(x),
            DWORD(x) => Ratio::from_u32(x),
            DINT(x) => Ratio::from_i32(x),
            REAL(x) => Ratio::from_f32(x),
            LREAL(x) => Ratio::from_f64(x),
            CHAR(x) => Ratio::from_u8(x),
            WCHAR(x) => Ratio::from_u16(x),
            _ => {
                return InterpreterResult::Err(String::from(
                    "Attempted to get number from a non-numeric type",
                ));
            }
        }
        .ok_or(String::from(
            "Could not convert number to rational representation",
        ))?,
    )
}

fn comparison_operation_result(
    left: VariableValue,
    op: &ComparisonOperator,
    right: VariableValue,
) -> InterpreterResult<VariableValue> {
    let left = get_num_from_variable(left)?;
    let right = get_num_from_variable(right)?;
    let comparison_result = match op {
        LessThan => left < right,
        LessEqualThan => left <= right,
        GreaterThan => left > right,
        GreaterEqualThan => left >= right,
    };
    InterpreterResult::Ok(VariableValue::BOOL(comparison_result))
}

fn math_operation_result(
    left: VariableValue,
    op: MathOp,
    right: VariableValue,
) -> InterpreterResult<VariableValue> {
    let left = get_num_from_variable(left)?;
    let right = get_num_from_variable(right)?;
    let math_result = match op {
        MathOp::Multiply(MultiplyOperator::MULTIPLY) => left.checked_mul(&right),
        MathOp::Multiply(MultiplyOperator::DIVIDE) => left.checked_div(&right),
        MathOp::Multiply(MultiplyOperator::MODULO) => {
            if left.is_integer() && right.is_integer() {
                Some(Ratio::from_integer(
                    left.to_integer().mod_floor(&right.to_integer()),
                ))
            } else {
                panic!("Attempted to take modulus of non-integral types");
            }
        }
        MathOp::Add(AddOperator::ADD) => left.checked_add(&right),
        MathOp::Add(AddOperator::SUBTRACT) => left.checked_sub(&right),
        MathOp::Exponentiate => {
            assert!(
                right.is_integer(),
                "Cannot exponentiate to non-integral powers"
            );
            let exponent = right.to_integer();
            let left = if exponent.is_positive() {
                left
            } else {
                left.recip()
            };
            let exponent = exponent
                .abs()
                .to_usize()
                .expect("Could not convert exponent to usize");
            checked_pow(left, exponent)
        }
    }
    .expect("Math result under/overflowed");
    let result = math_result
        .to_f32()
        .expect("Could not represent math result in internal format");
    InterpreterResult::Ok(VariableValue::REAL(result))
}
