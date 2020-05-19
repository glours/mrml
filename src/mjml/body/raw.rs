use crate::mjml::body::prelude::*;
use crate::mjml::body::BodyElement;
use crate::mjml::prelude::*;
use crate::mjml::{Component, Error};
use crate::util::prelude::PropertyMap;
use crate::util::{Attributes, Context};
use crate::{close_tag, open_tag};
use roxmltree::Node;
use std::collections::HashMap;

fn empty_str() -> String {
    "".into()
}

#[derive(Clone, Debug)]
pub struct CommentElement {
    content: String,
    context: Option<Context>,
}

impl Component for CommentElement {
    fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    fn set_context(&mut self, ctx: Context) {
        self.context = Some(ctx);
    }

    fn render(&self) -> Result<String, Error> {
        let keep_comments = self
            .context()
            .and_then(|ctx| Some(ctx.options().keep_comments))
            .or_else(|| Some(true))
            .unwrap();
        if !keep_comments || self.content.len() == 0 {
            return Ok(empty_str());
        }
        Ok(format!("<!--{}-->", self.content))
    }
}

#[derive(Clone, Debug)]
pub struct NodeElement {
    attributes: HashMap<String, String>,
    context: Option<Context>,
    children: Vec<BodyElement>,
    tag: String,
}

impl Component for NodeElement {
    fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    fn set_context(&mut self, ctx: Context) {
        self.context = Some(ctx);
    }

    fn render(&self) -> Result<String, Error> {
        let mut attrs = Attributes::new();
        for (key, value) in self.attributes.iter() {
            attrs.set(key, value);
        }
        let mut res = vec![];
        res.push(open_tag!(self.tag, attrs.to_string()));
        for child in self.children.iter() {
            res.push(child.render()?);
        }
        res.push(close_tag!(self.tag));
        Ok(res.join(""))
    }
}

impl ComponentWithAttributes for NodeElement {
    fn source_attributes(&self) -> Option<&HashMap<String, String>> {
        Some(&self.attributes)
    }
}

#[derive(Clone, Debug)]
pub enum RawElement {
    Comment(CommentElement),
    Node(NodeElement),
    Text(String),
}

fn get_node_text<'a, 'b>(node: Node<'a, 'b>) -> String {
    node.text()
        .and_then(|txt| Some(txt.to_string()))
        .or_else(|| Some(empty_str()))
        .unwrap()
}

impl RawElement {
    fn parse_comment<'a, 'b>(node: Node<'a, 'b>) -> Result<RawElement, Error> {
        Ok(RawElement::Comment(CommentElement {
            content: get_node_text(node),
            context: None,
        }))
    }

    fn parse_text<'a, 'b>(node: Node<'a, 'b>) -> Result<RawElement, Error> {
        Ok(RawElement::Text(get_node_text(node)))
    }

    fn parse_node<'a, 'b>(node: Node<'a, 'b>) -> Result<RawElement, Error> {
        let mut children = vec![];
        for child in node.children() {
            children.push(BodyElement::parse(child)?);
        }
        Ok(RawElement::Node(NodeElement {
            attributes: get_node_attributes(&node),
            context: None,
            children,
            tag: node.tag_name().name().to_string(),
        }))
    }

    pub fn parse<'a, 'b>(node: Node<'a, 'b>) -> Result<RawElement, Error> {
        if node.is_comment() {
            RawElement::parse_comment(node)
        } else if node.is_text() {
            RawElement::parse_text(node)
        } else {
            RawElement::parse_node(node)
        }
    }
}

impl Component for RawElement {
    fn context(&self) -> Option<&Context> {
        match self {
            RawElement::Comment(comment) => comment.context(),
            RawElement::Node(node) => node.context(),
            RawElement::Text(_) => None,
        }
    }

    fn set_context(&mut self, ctx: Context) {
        match self {
            RawElement::Comment(comment) => comment.set_context(ctx),
            RawElement::Node(node) => node.set_context(ctx),
            RawElement::Text(_) => (),
        };
    }

    fn render(&self) -> Result<String, Error> {
        match self {
            RawElement::Comment(comment) => comment.render(),
            RawElement::Node(node) => node.render(),
            RawElement::Text(content) => Ok(content.to_string()),
        }
    }
}

impl ComponentWithAttributes for RawElement {
    fn source_attributes(&self) -> Option<&HashMap<String, String>> {
        match self {
            RawElement::Node(node) => node.source_attributes(),
            _ => None,
        }
    }
}

impl BodyComponent for RawElement {}