use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::error::Error;
use crate::ApiServer;
//use api_tools::client::api_query::*;
//use api_tools::client::api_request::ApiRequest;


mod ship_general;
mod table;
mod frames_physical;
mod frames_theoretical;

pub use ship_general::*;
pub use table::*;
pub use frames_physical::*;
pub use frames_theoretical::*;

/// Класс-коллекция таблиц. Проверяет данные и выполняет их запись
pub struct Parser {
    data: String,
    api_server: Rc<RefCell<ApiServer>>,
    general: Option<General>,    
    parsed: HashMap<String, Box<dyn Table>>,
}
///
impl Parser {
    ///
    pub fn new(data: String, api_server: Rc<RefCell<ApiServer>>,) -> Self {
        Self {
            data,
            api_server,
            general: None,
            parsed: HashMap::<String, Box<dyn Table>>::new(),            
        }
    }
    /// Конвертация и проверка данных
    pub fn convert(&mut self) -> Result<(), Error> {
        println!("Parser convert begin");
        let json_data: serde_json::Value = serde_json::from_str(&self.data)?;
        //  println!("Data: {}", json_data);
        let fields = json_data
            .get("fields")
            .ok_or(Error::FromString(format!("No fields in data!")))?;
        //    println!("fields: {}", fields);
        let fields = fields
            .as_array()
            .ok_or(Error::FromString(format!("fields no array!")))?;
        for field in fields {
            let tag = field
                .get("tag")
                .ok_or(Error::FromString(format!("No tag, field:{field}")))?
                .as_str()
                .ok_or(Error::FromString(format!("Unknown tag in field:{field}")))?;
            let body = field
                .get("body")
                .ok_or(Error::FromString(format!("No body, field:{field}")))?
                .as_str()
                .ok_or(Error::FromString(format!("Unknown body in field:{field}")))?
                .to_owned();
            match tag {
                "general" => {
                    self.general = Some(General::new(body, Rc::clone(&self.api_server)));
                }
                text => {
                    self.parsed.insert(
                        text.to_owned(),
                        match text {
                            "frames_theoretical" => {
                                let mut table: Box::<dyn Table> = Box::new(TheoreticalFrame::new(body));
                                table.parse()?;
                                table
                            },
                            "frames_physical" => {
                                let mut table: Box::<dyn Table> = Box::new(PhysicalFrame::new(body));
                                table.parse()?;
                                table
                            },
                            _ => Err(Error::FromString(format!("Unknown tag: {text}")))?,
                        },
                    );
                }
            }
        }
        println!("Parser convert end");
        Ok(())
    }
    /// Запись данных в БД
    pub fn write_to_db(mut self) -> Result<(), Error> {
        println!("Parser write_to_db begin");
        let ship_id = self.general.take().ok_or(Error::FromString("Parser write_to_db error: no general".to_owned()))?.process()?;
        self.parsed.into_iter().for_each(|mut table| {
            if let Err(error) = self.api_server.borrow_mut().fetch(&table.1.to_sql(ship_id)) {
                println!("{}", format!("Parser write_to_db error:{}", error.to_string()));
            }
        });
        println!("Parser write_to_db end");
        Ok(())
    }
}
