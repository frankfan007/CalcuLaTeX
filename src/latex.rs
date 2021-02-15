use crate::expr::unit::BaseUnit;
use crate::expr::unit::BASE_UNITS;
use crate::expr::unit::UNIT_PREFIXES_ABBR;
use crate::expr::{unit::Unit, val::Val, Expr, Op};
use crate::unit_expr::UnitPow;
use num::rational::Ratio;
use num::One;
use num::Signed;
use num::Zero;

pub enum LaTeX {
    Text(String),
    Math(String),
}

#[derive(Debug)]
pub enum FormatArgs {
    UnitHint { string: String, value: UnitPow },
}

pub trait ToLaTeX {
    fn to_latex_ext(&self, args: Option<&FormatArgs>) -> LaTeX;
    fn to_latex(&self) -> LaTeX {
        self.to_latex_ext(None)
    }
}

impl ToString for LaTeX {
    fn to_string(&self) -> String {
        match self {
            LaTeX::Text(t) => t.to_owned(),
            LaTeX::Math(m) => m.to_string(),
        }
    }
}

impl ToLaTeX for Expr {
    fn to_latex_ext(&self, _: Option<&FormatArgs>) -> LaTeX {
        match self {
            Expr::Atom(v) => LaTeX::Math(v.to_string()),
            Expr::Ident(n) => LaTeX::Math(n.to_string()),
            Expr::Cons(op, e) => match (op, e.as_slice()) {
                (Op::Plus, [a, b, ..]) => LaTeX::Math(format!(
                    "({} + {})",
                    a.to_latex().to_string(),
                    b.to_latex().to_string()
                )),
                (Op::Minus, [a, b, ..]) => LaTeX::Math(format!(
                    "({} - {})",
                    a.to_latex().to_string(),
                    b.to_latex().to_string()
                )),
                (Op::Mul, [a, b, ..]) => LaTeX::Math(format!(
                    "{} \\times {}",
                    a.to_latex().to_string(),
                    b.to_latex().to_string()
                )),
                (Op::Div, [a, b, ..]) => LaTeX::Math(format!(
                    "\\frac{{{}}}{{{}}}",
                    a.to_latex().to_string(),
                    b.to_latex().to_string()
                )),
                (Op::Exp, [a, b, ..]) => LaTeX::Math(format!(
                    "{}^{{{}}}",
                    a.to_latex().to_string(),
                    b.to_latex().to_string()
                )),
                (Op::AddMultiUnit(m, u), [v]) => LaTeX::Math(format!(
                    "{}\\ {}{}",
                    v.to_latex().to_string(),
                    UNIT_PREFIXES_ABBR.get_by_right(m).unwrap_or(&""),
                    u.to_latex().to_string(),
                )),
                _ => todo!(),
            },
        }
    }
}

impl ToLaTeX for Val {
    fn to_latex_ext(&self, args: Option<&FormatArgs>) -> LaTeX {
        match args {
            Some(FormatArgs::UnitHint {
                string,
                value: UnitPow { unit, pow },
            }) if unit == &self.unit => {
                let out = format!("{} \\ {}", self.num / 10f64.powi(*pow as i32), string);
                LaTeX::Math(out.trim().to_string())
            }
            Some(FormatArgs::UnitHint { string, .. }) => {
                panic!(
                    "Unit hint {} does not value with unit {}",
                    string, self.unit
                )
            }
            None => {
                let unit_str = self.unit.to_latex().to_string();
                let out = if !unit_str.is_empty() {
                    format!("{} \\ {}", self.num, unit_str)
                } else {
                    self.num.to_string()
                };
                LaTeX::Math(out.trim().to_string())
            }
        }
    }
}

impl ToLaTeX for Unit {
    fn to_latex_ext(&self, _: Option<&FormatArgs>) -> LaTeX {
        match self {
            Unit::Base(arr) => {
                let mut numerator = Vec::new();
                let mut denominator = Vec::new();
                arr.iter().zip(BASE_UNITS.iter()).for_each(|(pow, unit)| {
                    use std::cmp::Ordering::*;

                    match pow.cmp(&Ratio::zero()) {
                        Greater => numerator.push((pow, unit)),
                        Less => denominator.push((pow, unit)),
                        _ => {}
                    }
                });

                let latexify_single_unit = |(pow, unit): &(&Ratio<i8>, &BaseUnit)| {
                    if pow.abs() == Ratio::one() {
                        unit.to_string()
                    } else {
                        format!("{}^{{{}}}", unit.to_string(), pow.abs())
                    }
                };

                let numerator_string = numerator.iter().fold("".to_string(), |acc, unit_info| {
                    format!("{} {}", acc, latexify_single_unit(unit_info))
                });
                let denominator_string =
                    denominator.iter().fold("".to_string(), |acc, unit_info| {
                        format!("{} {}", acc, latexify_single_unit(unit_info))
                    });

                if numerator_string.is_empty() {
                    LaTeX::Math(format!("\\frac{{1}}{{{}}}", denominator_string))
                } else if denominator.is_empty() {
                    LaTeX::Math(numerator_string)
                } else {
                    LaTeX::Math(format!(
                        "\\frac{{{}}}{{{}}}",
                        numerator_string, denominator_string
                    ))
                }
            }
            Unit::Custom(_map) => {
                todo!()
            }
        }
    }
}
