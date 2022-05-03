use std::fmt::{self, Debug, Formatter};

use super::{Content, Interruption, NodeId, Show, ShowNode, StyleEntry};
use crate::diag::{At, TypResult};
use crate::eval::{Args, Func, Value};
use crate::library::structure::{EnumNode, ListNode};
use crate::syntax::Span;
use crate::Context;

/// A show rule recipe.
#[derive(Clone, PartialEq, Hash)]
pub struct Recipe {
    /// The patterns to customize.
    pub pattern: Pattern,
    /// The function that defines the recipe.
    pub func: Func,
    /// The span to report all erros with.
    pub span: Span,
}

impl Recipe {
    /// Whether the recipe is applicable to the target.
    pub fn applicable(&self, target: Target) -> bool {
        match (&self.pattern, target) {
            (Pattern::Node(id), Target::Node(node)) => *id == node.id(),
            _ => false,
        }
    }

    /// Try to apply the recipe to the target.
    pub fn apply(
        &self,
        ctx: &mut Context,
        sel: Selector,
        target: Target,
    ) -> TypResult<Option<Content>> {
        let content = match (target, &self.pattern) {
            (Target::Node(node), &Pattern::Node(id)) if node.id() == id => {
                let node = node.unguard(sel);
                self.call(ctx, || {
                    let dict = node.encode();
                    Value::Content(Content::Show(node, Some(dict)))
                })?
            }

            _ => return Ok(None),
        };

        Ok(Some(content.styled_with_entry(StyleEntry::Guard(sel))))
    }

    /// Call the recipe function, with the argument if desired.
    fn call<F>(&self, ctx: &mut Context, arg: F) -> TypResult<Content>
    where
        F: FnOnce() -> Value,
    {
        let args = if self.func.argc() == Some(0) {
            Args::new(self.span)
        } else {
            Args::from_values(self.span, [arg()])
        };

        self.func.call(ctx, args)?.cast().at(self.span)
    }

    /// What kind of structure the property interrupts.
    pub fn interruption(&self) -> Option<Interruption> {
        if let Pattern::Node(id) = self.pattern {
            if id == NodeId::of::<ListNode>() || id == NodeId::of::<EnumNode>() {
                return Some(Interruption::List);
            }
        }

        None
    }
}

impl Debug for Recipe {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Recipe matching {:?} from {:?}", self.pattern, self.span)
    }
}

/// A show rule pattern that may match a target.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Pattern {
    /// Defines the appearence of some node.
    Node(NodeId),
}

/// A target for a show rule recipe.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Target<'a> {
    /// A showable node.
    Node(&'a ShowNode),
}

/// Identifies a show rule recipe.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum Selector {
    /// The nth recipe from the top of the chain.
    Nth(usize),
    /// The base recipe for a kind of node.
    Base(NodeId),
}