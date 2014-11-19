/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::comm::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::collections::TreeMap;

use servo_util::str::DOMString;
use servo_util::task::spawn_named;

pub enum StorageTaskMsg {
    // Request the storage data associated with a particular origin
    Length(Sender<u32>, String),
    Key(Sender<Option<DOMString>>, String, u32),
    GetItem(Sender<Option<DOMString>>, String, DOMString),
    SetItem(String, DOMString, DOMString),
    RemoveItem(String, DOMString),
    Clear(String),
    Exit
}

// Handle to a storage task
pub type StorageTask = Sender<StorageTaskMsg>;

// Create a StorageTask
pub fn new_storage_task() -> StorageTask {
    let (chan, port) = channel();
    spawn_named("StorageManager", proc() {
        StorageManager::new(port).start();
    });
    chan
}

struct StorageManager {
    port: Receiver<StorageTaskMsg>,
    data: RefCell<HashMap<String, RefCell<TreeMap<DOMString, DOMString>>>>,
}

impl StorageManager {
    fn new(port: Receiver<StorageTaskMsg>) -> StorageManager {
        StorageManager {
            port: port,
            data: RefCell::new(HashMap::new()),
        }
    }
}

impl StorageManager {
    fn start(&self) {
        loop {
            match self.port.recv() {
              Length(sender, origin) => {
                  self.length(sender, origin)
              }
              Key(sender, origin, index) => {
                  self.key(sender, origin, index)
              }
              SetItem(origin, name, value) => {
                  self.set_item(origin, name, value)
              }
              GetItem(sender, origin, name) => {
                  self.get_item(sender, origin, name)
              }
              RemoveItem(origin, name) => {
                  self.remove_item(origin, name)
              }
              Clear(origin) => {
                  self.clear(origin)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn length(&self, sender: Sender<u32>, origin: String) {
        match self.data.borrow().get(&origin) {
            Some(origin_data) => sender.send(origin_data.borrow().len() as u32),
            None => sender.send(0),
        }
    }

    fn key(&self, sender: Sender<Option<DOMString>>, origin: String, index: u32) {
        let mut result: Option<DOMString> = None;

        match self.data.borrow().get(&origin) {
            Some(origin_data) => {
                if index < origin_data.borrow().len() as u32 {
                    let mut i: u32 = 0;
                    for key in origin_data.borrow().keys() {
                        if i == index {
                            result = Some((*key).clone());
                            break;
                        }
                        i = i + 1;
                    }

                }
            }
            None => {}
        }

        sender.send(result);
    }

    fn set_item(&self,  origin: String, name: DOMString, value: DOMString) {
        if !self.data.borrow().contains_key(&origin) {
            self.data.borrow_mut().insert(origin.clone(), RefCell::new(TreeMap::new()));
        }

        match self.data.borrow().get(&origin) {
            Some(origin_data) => {
                origin_data.borrow_mut().insert(name, value);
            }
            None => {}
        }
    }

    fn get_item(&self, sender: Sender<Option<DOMString>>, origin: String, name: DOMString) {
        let mut result: Option<DOMString> = None;

        match self.data.borrow().get(&origin) {
            Some(origin_data) => {
                match origin_data.borrow().get(&name) {
                    Some(value) => result = Some(value.to_string()),
                    None => {},
                }
            }
            None => {}
        }

        sender.send(result);
    }

    fn remove_item(&self,  origin: String, name: DOMString) {
        match self.data.borrow().get(&origin) {
            Some(origin_data) => {
                origin_data.borrow_mut().remove(&name);
            }
            None => {}
        }
    }

    fn clear(&self, origin: String) {
        match self.data.borrow().get(&origin) {
            Some(origin_data) => origin_data.borrow_mut().clear(),
            None => {}
        }
    }
}
