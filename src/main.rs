mod vimdoc;
use vimdoc::{Document, Function};
use xmltree::Element;
use xmltree::XMLNode;
use std::fs::File;
use std::env;
use std::path::PathBuf;

use crate::vimdoc::Class;

fn get_enum(e: &Element, parentns: &str) -> Vec<String> {
    let mut ret = vec![];
    let ns = &e.attributes["name"];
    for child in e.children.iter() {
        match child {
            XMLNode::Element(e) =>  {
                if e.name == "member" {
                   ret.push(format!("{}.{}.{}", parentns, ns, e.attributes["name"].to_uppercase()))
                }
            }
            _ => {}
        }
    }
    ret
}

// move this to the lua/language
fn translate(str: &str, ns: &str) -> String {
    match str {
        "utf8"|"const char*"|"char*" => "string".to_string(),
        "gboolean" => "boolean".to_string(),
        "glong"|"gssize"|"gint64"|"gint"|"gsize"|"guint32" => "num".to_string(),
        "none" => "nil".to_string(),
        rest => {
            if !rest.contains(".") {
                return format!("{}.{}", ns, str)
            }
            rest.to_string()
        }
    }
}

// both of these are kinda wrong, because error values etc
fn get_return(e: &Element, ns: &str) -> Option<Vec<String>> {
    let ret = e.get_child("return-value")?;
    if let Some(t) = ret.get_child("type") {
        let name = &t.attributes.get("name")?;
        return Some(vec![name.to_string()])
        // return Some(vec![translate(name, ns)])
    }
    if let Some(t) = ret.get_child("array") {
        let name = &t.attributes.get("name")?;
        return Some(vec![name.to_string()])
        // return Some(vec![translate(name, ns)])
    }
    None
}

// this should be more general and not equal for
// all languages, and shouldn't be done here?
fn filter(typ: &String) -> bool {
    match typ.as_ref() {
        "gpointer" => true,
        _ => false,
    }
}


// TODO a way to push arguments to return values, when a argument is returned
// and not passed as an argument. Example a function apa(int a, err **Error) 
// becomes apa(int) -> Error

fn get_params(e: &Element, ns: &str) -> Option<Vec<(String, String)>> {
    let mut ret: Vec<(String, String)> = vec![];
    let parameters = e.get_child("parameters")?;

    for parameter in parameters.children.iter() {
        match parameter {
            XMLNode::Element(element) => {
                if element.name == "parameter" {
                    let argname = element.attributes.get("name")?.clone();
                    let e2 = element.get_child("type")?;
                    let argtype = &e2.attributes.get("name")?;
                    if !filter(&argtype) {
                        ret.push((argname, translate(argtype, ns)));
                    }
                }
            }
            _ => {}
        }
    }
    return Some(ret)
}

fn get_inner_doc(e: &Element) -> Option<String> {
    e.get_child("doc").map(|doc| get_doc(doc)).flatten()
}

fn callable(e: &Element, parentns: &str) -> Option<Function> {
    let name = e.attributes.get("name")?;
    let intro = e.attributes.get("introspectable");

    // TODO should be possible to do this
    if let Some(enabl) = intro {
        if enabl == "0" {
            return None
        }
    }
    let doc = get_inner_doc(&e);
    let ret = get_return(&e, parentns).unwrap_or(vec![]);
    let args = get_params(&e, parentns).unwrap_or(vec![]);
    Some(Function::new(name.to_string(), doc, args, ret))
}

fn get_doc(e: &Element) -> Option<String> {
    e.get_text().map(|x| x.into())
}

fn get_field(e: &Element) -> Option<(String, String)> {
    let name = e.attributes.get("name")?;
    let doc = get_inner_doc(e)?;
    Some((name.to_string(), doc))
}

fn get_class(parentns: &str, e: &Element) -> Class {
    let ns = &e.attributes["name"];
    let mut class = Class::new(ns);
    for node in e.children.iter() {
        match node {
            XMLNode::Element(ref e) => {
                match e.name.as_str() {
                    "doc" => {
                        let docs = get_doc(&e);
                        class.set_docs(docs);
                    }
                    "field" => {
                        if let Some(field) = get_field(e) {
                            class.add_field(field)
                        }
                    }
                    "source-position" => {
                    }
                    "constructor" => {
                        if let Some(fun) = callable(e, parentns) {
                            class.add_constructor(fun)
                        }
                    }
                    "function" => {
                        if let Some(fun) = callable(e, parentns) {
                            class.add_function(fun)
                        }
                    }
                    "method" => {
                        if let Some(fun) = callable(e, parentns) {
                            class.add_method(fun)
                        }
                    }
                    "virtual-method" => { 
                        if let Some(fun) = callable(e, parentns) {
                            class.add_virtual(fun)
                        }
                    }
                    "property" => {
                        // println!("{:#?}", e)
                    }
                    "signal" => {
                        // println!("{:#?}", e)
                        // TODO
                    }
                    "implements" => {
                        // TODO
                    }
                    name => {
                        panic!("Name: {} not matched against\n", name)
                    }
                }
            }
            _ => {}
        }
    }
    class
}

fn get_macro(e: &Element) {
}

// add namespace
fn get_global(doc: &mut Document, node: &XMLNode) {
    match node {
        XMLNode::Element(ref e) => {
            // print classes first,
            // then print functions,
            // then macros
            // then print the rest
            match e.name.as_str() {
                "class" => {
                    let class = get_class(&doc.ns, e);
                    doc.add_class(class);
                }
                "function" => {
                    if let Some(fun) = callable(e, &doc.ns) {
                        doc.add_function(fun)
                    }
                }
                "function-macro" => {
                    if let Some(fun) = callable(e, &doc.ns) {
                        doc.add_macro(fun)
                    }
                }
                "enumeration" => {
                    for enu in get_enum(&e, &doc.ns).into_iter() {
                        doc.add_enum(enu);
                    }
                }
                "record" => {
                    // println!("{:#?}", e)
                    // is-gtype-struct-for ? Is that the dynamic type check function?
                }
                "constant" => {
                    // println!("{:#?}", e)
                }
                "callback" => {
                    // TODO
                }
                "bitfield" => {
                    // println!("{:#?}", e)
                }
                "docsection" => {
                    println!("{:#?}", e)
                }
                "name" => {
                }
                "alias" => {
                }
                "interface" => {
                }
                "boxed" => {
                }
                name => {
                    panic!("Name: {} not matched against\n", name)
                }
            }
        }
        _ => {}
    }
}

fn parse_toplevel(node: &XMLNode) -> Option<Document> {
    match node {
        XMLNode::Element(ref e) => {
            if e.name == "namespace" {
                let ns = &e.attributes["name"];
                let mut doc = Document::new(ns,
                    "",
                    "");
                for node in e.children.iter() {
                    get_global(&mut doc, node)
                }
                return Some(doc);
            }
            None
        }
        _ => None
    }
}

fn main() {
    for arg in env::args().skip(1) {
        let f = File::open(&arg).expect("Can't read file");
        let mut out_file = PathBuf::from(&arg);
        out_file.set_extension("txt");
        let mut out = File::create(out_file).expect("Couldn't open output file");

        let names_element = Element::parse(f).unwrap();

        for child in names_element.children.into_iter() {
            let doc = parse_toplevel(&child);
            if let Some(doc) = doc {
                doc.write(&mut out).expect("Couldn't write document to output file");
            }
        }
    }
}
