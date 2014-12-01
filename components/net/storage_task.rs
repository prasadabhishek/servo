/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::comm::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::collections::TreeMap;
use url::Url;

use servo_util::str::DOMString;
use servo_util::task::spawn_named;

/// Request operations on the storage data associated with a particular url
pub enum StorageTaskMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(Sender<u32>, Url),

    /// gets the name of the key at the specified index in the associated storage data
    Key(Sender<Option<DOMString>>, Url, u32),

    /// gets the value associated with the given key in the associated storage data
    GetItem(Sender<Option<DOMString>>, Url, DOMString),

    /// sets the value of the given key in the associated storage data
    /// TODO throw QuotaExceededError in case of error
    SetItem(Url, DOMString, DOMString),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(Url, DOMString),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(Url),

    /// shut down this task
    Exit
}

/// Handle to a storage task
pub type StorageTask = Sender<StorageTaskMsg>;

/// Create a StorageTask
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
              Length(sender, url) => {
                  self.length(sender, url)
              }
              Key(sender, url, index) => {
                  self.key(sender, url, index)
              }
              SetItem(url, name, value) => {
                  self.set_item(url, name, value)
              }
              GetItem(sender, url, name) => {
                  self.get_item(sender, url, name)
              }
              RemoveItem(url, name) => {
                  self.remove_item(url, name)
              }
              Clear(url) => {
                  self.clear(url)
              }
              Exit => {
                break
              }
            }
        }
    }

    fn length(&self, sender: Sender<u32>, url: Url) {
        let origin = self.get_origin_as_string(url);
        match self.data.borrow().get(&origin) {
            Some(origin_data) => sender.send(origin_data.borrow().len() as u32),
            None => sender.send(0),
        }
    }

    fn key(&self, sender: Sender<Option<DOMString>>, url: Url, index: u32) {
        let mut result: Option<DOMString> = None;

        let origin = self.get_origin_as_string(url);
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

    fn set_item(&self,  url: Url, name: DOMString, value: DOMString) {
        let origin = self.get_origin_as_string(url);
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

    fn get_item(&self, sender: Sender<Option<DOMString>>, url: Url, name: DOMString) {
        let mut result: Option<DOMString> = None;

        let origin = self.get_origin_as_string(url);
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

    fn remove_item(&self, url: Url, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        match self.data.borrow().get(&origin) {
            Some(origin_data) => {
                origin_data.borrow_mut().remove(&name);
            }
            None => {}
        }
    }

    fn clear(&self, url: Url) {
        let origin = self.get_origin_as_string(url);
        match self.data.borrow().get(&origin) {
            Some(origin_data) => origin_data.borrow_mut().clear(),
            None => {}
        }
    }

    fn get_origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_string();
        origin.push_str(url.scheme.as_slice());
        origin.push_str("://");
        url.domain().map(|domain| origin.push_str(domain.as_slice()));
        origin.push_str(":");
        url.port().map(|port| origin.push_str(port.to_string().as_slice()));
        origin.push_str("/");
        origin
    }
}
