use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

use super::css::Styleable;
use super::html::Tag;
use gtk::prelude::*;
use mlua::prelude::*;

use mlua::{Lua, LuaSerdeExt, OwnedFunction, Value};

use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Map;

lazy_static! {
    static ref LUA_LOGS: Mutex<String> = Mutex::new(String::new());
}

macro_rules! problem {
    ($type:expr, $s:expr) => {{
        let problem_type = match ($type) {
            "error" => "ERROR",
            "warning" => "WARNING",
            _ => "UNKNOWN",
        };

        let log_msg = format!("{}: {}\n", problem_type, $s);

        if let Ok(mut lua_logs) = LUA_LOGS.lock() {
            lua_logs.push_str(&log_msg);
        } else {
            eprintln!("FATAL: failed to lock lua logs mutex!");
        }
    }};
}

pub trait Luable: Styleable {
    fn get_css_classes(&self) -> Vec<String>;
    fn get_css_name(&self) -> String;

    fn get_contents(&self) -> String;
    fn get_href(&self) -> String;

    fn set_contents(&self, contents: String);
    fn set_href(&self, href: String);

    fn _on_click<'a>(&self, func: &'a LuaOwnedFunction);
    fn _on_submit<'a>(&self, func: &'a LuaOwnedFunction);
    fn _on_input<'a>(&self, func: &'a LuaOwnedFunction);
}
// use tokio::time::{sleep, Duration};

// async fn sleep_ms(_lua: &Lua, ms: u64) -> LuaResult<()> {
//     sleep(Duration::from_millis(ms)).await;
//     Ok(())
// }

fn get<'lua>(
    lua: &'lua Lua,
    class: String,
    tags: Rc<RefCell<Vec<Tag>>>,
) -> LuaResult<LuaTable<'lua>> {
    let tags_ref = tags.borrow();

    for (i, tag) in tags_ref.iter().enumerate() {
        if tag.classes.contains(&class) {
            let tags1 = Rc::clone(&tags);
            let tags2 = Rc::clone(&tags);
            let tags3 = Rc::clone(&tags);
            let tags4 = Rc::clone(&tags);
            let tags5 = Rc::clone(&tags);
            let tags6 = Rc::clone(&tags);
            let tags7 = Rc::clone(&tags);

            let table = lua.create_table()?;

            let css_name = tag.widget.get_css_name().clone();

            table.set("name", css_name)?;

            table.set(
                "get_content",
                lua.create_function(move |_, ()| {
                    let ok = tags1.borrow()[i].widget.get_contents();
                    Ok(ok)
                })?,
            )?;
            table.set(
                "set_content",
                lua.create_function(move |_, label: String| {
                    tags2.borrow()[i].widget.set_contents(label);
                    Ok(())
                })?,
            )?;
            table.set(
                "on_click",
                lua.create_function(move |_lua, func: OwnedFunction| {
                    tags3.borrow()[i].widget._on_click(&func);
                    Ok(())
                })?,
            )?;
            table.set(
                "on_submit",
                lua.create_function(move |_lua, func: OwnedFunction| {
                    tags4.borrow()[i].widget._on_submit(&func);
                    Ok(())
                })?,
            )?;
            table.set(
                "on_input",
                lua.create_function(move |_lua, func: OwnedFunction| {
                    tags5.borrow()[i].widget._on_input(&func);
                    Ok(())
                })?,
            )?;
            table.set(
                "get_href",
                lua.create_function(move |_, ()| {
                    let ok = tags6.borrow()[i].widget.get_href();
                    Ok(ok)
                })?,
            )?;
            table.set(
                "set_href",
                lua.create_function(move |_, label: String| {
                    tags7.borrow()[i].widget.set_href(label);
                    Ok(())
                })?,
            )?;

            return Ok(table);
        }
    }

    Err(LuaError::RuntimeError("Tag not found".into()))
}

fn print(_lua: &Lua, msg: LuaMultiValue) -> LuaResult<()> {
    let mut output = String::new();
    for value in msg.iter() {
        match value {
            Value::String(s) => output.push_str(s.to_str().unwrap_or("")),
            Value::Integer(i) => output.push_str(&i.to_string()),
            Value::Number(n) => output.push_str(&n.to_string()),
            Value::Boolean(b) => output.push_str(&b.to_string()),
            def => output.push_str(&format!("{def:#?}")),
        }
    }

    println!("{}", output);
    Ok(())
}

// todo: make this async if shit breaks
pub(crate) async fn run(luacode: String, tags: Rc<RefCell<Vec<Tag>>>) -> LuaResult<()> {
    let lua = Lua::new();
    let globals = lua.globals();

    let fetchtest = lua.create_async_function(|lua, params: LuaTable| async move {
        // I LOVE MATCH STATEMENTSI LOVE MATCH STATEMENTSI LOVE MATCH STATEMENTSI LOVE MATCH STATEMENTSI LOVE MATCH STATEMENTSI LOVE MATCH STATEMENTS
        let uri = match params.get::<_, String>("url") {
            Ok(url) => url,
            Err(_) => return Err(LuaError::RuntimeError("url is required".into())),
        };
        let method = match params.get::<_, String>("method") {
            Ok(method) => method,
            Err(_) => return Err(LuaError::RuntimeError("method is required".into())),
        };
        let headers = match params.get::<_, LuaTable>("headers") {
            Ok(headers) => headers,
            Err(_) => return Err(LuaError::RuntimeError("headers is required".into())),
        };

        let body_str = match params.get::<_, String>("body") {
            Ok(body) => body,
            Err(_) => "{}".to_string(),
        };

        let mut headermap = HeaderMap::new();

        for header in headers.pairs::<String, String>() {
            let (key, value) = header.unwrap_or(("".to_string(), "".to_string()));

            headermap.insert(
                HeaderName::from_bytes(key.as_ref()).unwrap(),
                HeaderValue::from_str(&value).unwrap(),
            );
        }

        let handle = thread::spawn(move || {
            let client = reqwest::blocking::Client::new();

            let req = match method.as_str() {
                "GET" => client.get(uri).headers(headermap),
                "POST" => client.post(uri).headers(headermap).body(body_str),
                "PUT" => client.put(uri).headers(headermap).body(body_str),
                "DELETE" => client.delete(uri).headers(headermap).body(body_str),
                _ => return format!("Unsupported method: {}", method).into(),
            };

            let res = match req.send() {
                Ok(res) => res,
                Err(e) => {
                    return format!("Failed to send request: {}", e).into();
                }
            };

            let errcode = Rc::new(RefCell::new(res.status().as_u16()));

            let body: Result<serde_json::Value, reqwest::Error> = res.json();

            let result = match body {
                Ok(body) => body,
                Err(e) => {
                    let errcode_clone = Rc::clone(&errcode);

                    problem!("error", format!("failed to parse response body: {}", e));
                    let mut map = Map::new();

                    map.insert("status".to_owned(), serde_json::Value::Number(serde_json::Number::from_f64(*errcode_clone.borrow() as f64).unwrap()));
                                        
                    serde_json::Value::Object(map)
                }
            };

            result
        });

        let json = match handle.join() {
            Ok(json) => json,
            Err(_) => {
                problem!(
                    "error",
                    format!("Failed to join request thread at fetch request.")
                );
                serde_json::Value::Null
            }
        };

        lua.to_value(&json)
    })?;

    globals.set("print", lua.create_function(print)?)?;
    globals.set(
        "get",
        lua.create_function(move |lua, class: String| get(lua, class, tags.clone()))?,
    )?;
    globals.set("fetch", fetchtest)?;

    let ok = lua.load(luacode).eval::<LuaMultiValue>();

    match ok {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "--------------------------\nerror: {}\n--------------------------------",
                e
            );
            Err(LuaError::runtime("Failed to run script!"))
        }
    }
}

// UTILS
fn gtk_buffer_to_text(buffer: &gtk::TextBuffer) -> String {
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, true);
    text.to_string()
}
// IMPLEMENTATIONS

// h1, h2, h3, h4, h5, h6
impl Luable for gtk::Label {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        self.text().to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "Most text-based components do not support the \"get_href\" method. Are you perhaps looking for the \"p\" tag?"
        );
        "".to_string()
    }

    fn set_contents(&self, contents: String) {
        self.set_text(&contents);
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "Most text-based components do not support the \"set_href\" method. Are you perhaps looking for the \"p\" tag?"
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "Text-based components do not support the \"submit\" event. Are you perhaps looking for the \"click\" event?"
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "Text-based components do not support the \"input\" event."
        );
    }
}

// select
impl Luable for gtk::DropDown {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        "".to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"select\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, _: String) {
        problem!(
            "warning",
            "\"select\" component does not support the \"set_content\" method."
        );
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"select\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"select\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"select\" component does not support the \"input\" event."
        );
    }
}

// a
impl Luable for gtk::LinkButton {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        self.label().unwrap_or("".into()).to_string()
    }
    fn get_href(&self) -> String {
        self.uri().to_string()
    }

    fn set_contents(&self, contents: String) {
        self.set_label(&contents);
    }
    fn set_href(&self, href: String) {
        self.set_uri(&href);
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"a\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"a\" component does not support the \"input\" event."
        );
    }
}

// div
impl Luable for gtk::Box {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        "".to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"div\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, _: String) {
        problem!(
            "warning",
            "\"div\" component does not support the \"set_content\" method."
        );
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"div\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"div\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"div\" component does not support the \"input\" event."
        );
    }
}

// textarea
impl Luable for gtk::TextView {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        let buffer = self.buffer();
        gtk_buffer_to_text(&buffer)
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"textarea\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, contents: String) {
        self.buffer().set_text(&contents);
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"textarea\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"textarea\" component does not support the \"submit\" event. Are you perhaps looking for the \"input\" event?"
        )
    }
    fn _on_input<'a>(&self, func: &'a LuaOwnedFunction) {
        let a = Rc::new(func.clone());

        self.buffer().connect_changed(move |s| {
            if let Err(e) = a.call::<_, ()>(gtk_buffer_to_text(s)) {
                problem!("error", e.to_string());
            }
        });
    }
}

// hr
impl Luable for gtk::Separator {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        "".to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"hr\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, _: String) {
        problem!(
            "warning",
            "\"hr\" component does not support the \"set_content\" method."
        );
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"hr\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"hr\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"hr\" component does not support the \"input\" event."
        );
    }
}

// img
impl Luable for gtk::Picture {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        "".to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"img\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, _: String) {
        problem!(
            "warning",
            "\"img\" component does not support the \"set_content\" method."
        );
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"img\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"img\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"img\" component does not support the \"input\" event."
        );
    }
}

// input
impl Luable for gtk::Entry {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        self.text().to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"input\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, contents: String) {
        self.buffer().set_text(&contents);
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"input\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let gesture = gtk::GestureClick::new();

        let a = Rc::new(func.clone());

        gesture.connect_released(move |_, _, _, _| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });

        self.add_controller(gesture)
    }
    fn _on_submit<'a>(&self, func: &'a LuaOwnedFunction) {
        let a = Rc::new(func.clone());

        self.connect_activate(move |entry| {
            let content = entry.buffer().text().to_string();

            if let Err(e) = a.call::<_, ()>(content) {
                problem!("error", e.to_string());
            }
        });
    }
    fn _on_input<'a>(&self, func: &'a LuaOwnedFunction) {
        let a = Rc::new(func.clone());

        self.connect_changed(move |entry| {
            let content = entry.buffer().text().to_string();

            if let Err(e) = a.call::<_, ()>(content) {
                problem!("error", e.to_string());
            }
        });
    }
}

// button
impl Luable for gtk::Button {
    fn get_css_classes(&self) -> Vec<String> {
        self.css_classes().iter().map(|s| s.to_string()).collect()
    }

    fn get_css_name(&self) -> String {
        self.css_name().to_string()
    }

    fn get_contents(&self) -> String {
        self.label().unwrap_or("".into()).to_string()
    }
    fn get_href(&self) -> String {
        problem!(
            "warning",
            "\"button\" component does not support the \"get_href\" method."
        );
        "".to_string()
    }

    fn set_contents(&self, contents: String) {
        self.set_label(&contents);
    }
    fn set_href(&self, _: String) {
        problem!(
            "warning",
            "\"button\" component does not support the \"set_href\" method."
        );
    }

    fn _on_click(&self, func: &LuaOwnedFunction) {
        let a = Rc::new(func.clone());

        self.connect_clicked(move |_| {
            if let Err(e) = a.call::<_, ()>(LuaNil) {
                problem!("error", e.to_string());
            }
        });
    }
    fn _on_submit<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"button\" component does not support the \"submit\" event."
        );
    }
    fn _on_input<'a>(&self, _: &'a LuaOwnedFunction) {
        problem!(
            "warning",
            "\"button\" component does not support the \"input\" event."
        );
    }
}
