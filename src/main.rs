mod vimdoc;
use vimdoc::{Document, Function};
use xmltree::Element;
use xmltree::XMLNode;
use std::collections::HashMap as AttributeMap;
// use xmltree::AttributeMap;
use std::fs::File;
use std::env;

use crate::vimdoc::Class;

    
fn print_header(attribs: &AttributeMap<String, String>) {
    // println!("{:#?}", attribs);
}

fn print_entry_type(attribs: &AttributeMap<String, String>) {
    // println!("{:#?}", attribs["name"]);
}

fn get_enum(e: &Element, parentns: &str) -> Option<String> {
    let ns = &e.attributes["name"];
    for child in e.children.iter() {
        match child {
            XMLNode::Element(e) =>  {
                if e.name == "member" {
                   return Some(format!("{}.{}.{}", parentns, ns, e.attributes["name"].to_uppercase()))
                }
            }
            _ => {}
        }
    }
    None
}

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
    let e2 = e.get_child("return-value")?;
    if let Some(t) = e2.get_child("type") {
        let name = &t.attributes.get("name")?;
        return Some(vec![translate(name, ns)])
    }
    if let Some(t) = e2.get_child("array") {
        let name = &t.attributes.get("name")?;
        return Some(vec![translate(name, ns)])
    }
    None
}

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
    let e2 = e.get_child("parameters")?;
    for child in e2.children.iter() {
        match child {
            XMLNode::Element(e) => {
                if e.name == "parameter" {
                    let argname = e.attributes["name"].clone();
                    if let Some(e2) = e.get_child("type") {
                        let argtype = &e2.attributes.get("name")?;
                        if !filter(&argtype) {
                            ret.push((argname, translate(argtype, ns)));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    return Some(ret)
}

fn get_doc2(e: &Element) -> Option<String> {
    if let Some(doc) = e.get_child("doc") {
        let txt = doc.get_text()?;
        return Some(txt.into_owned())
    }
    None
}


fn callable(e: &Element, parentns: &str) -> Option<Function> {
    let name = e.attributes["name"];
    let intro = e.attributes.get("introspectable");
    if let Some(enabl) = intro {
        if enabl == "0" {
            return None
        }
    }
    // What happens params arguments = 0? Does gir add 0 args? Same for ret
    let args = get_params(&e, parentns)?;
    let ret = get_return(&e, parentns)?;
    let doc = get_doc2(&e);
    Some(Function {
        name,
        doc,
        args,
        ret
    })
}

fn get_doc(e: &Element) -> Option<String> {
    e.get_text().map(|x| x.into())
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
                        // TODO
                        // println!("{:#?}", e)
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
                        // TODO
                    }
                    "signal" => {
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

fn print_macro(e: &Element) {
    // println!("{:#?}", e);
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
                    // TODO
                    if let Some(enu) = get_enum(&e, &doc.ns) {
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
                    // println!("{:#?}", e)
                }
                "bitfield" => {
                    // println!("{:#?}", e)
                }
                "docsection" => {
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

fn parse_toplevel(node: &XMLNode) {
    match node {
        XMLNode::Element(ref e) => {
            if e.name == "namespace" {
                let ns = &e.attributes["name"];
                let mut doc = Document::new(ns,
                    "gtk.txt {gir generated documentation for gtk}",
                    "oau");
                // print_header(&e.attributes);
                for node in e.children.iter() {
                    get_global(&mut doc, node)
                }
            }
        }
        _ => {}
    }
}

fn main() {
    for arg in env::args().skip(1) {
        let f = File::open(arg).expect("Can't read file");

        let names_element = Element::parse(f).unwrap();

        // for name in names_element {
        //
        // }
        for child in names_element.children.into_iter() {
            parse_toplevel(&child);
        }
        {
            // get first `name` element
            // name.attributes.insert("suffix".to_owned(), "mr".to_owned());
        }
    }
    // names_element.write(File::create("result.xml").unwrap());

}
