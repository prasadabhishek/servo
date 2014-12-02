/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
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
    data: HashMap<String, TreeMap<DOMString, DOMString>>,
}

impl StorageManager {
    fn new(port: Receiver<StorageTaskMsg>) -> StorageManager {
        StorageManager {
            port: port,
            data: HashMap::new(),
        }
    }
}

impl StorageManager {
    fn start(&mut self) {
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
        match self.data.get(&origin) {
            Some(origin_data) => sender.send(origin_data.len() as u32),
            None => sender.send(0),
        }
    }

    fn key(&self, sender: Sender<Option<DOMString>>, url: Url, index: u32) {
        let origin = self.get_origin_as_string(url);
        let result = self.data.get(&origin).
            and_then(|entry| entry.keys().nth(index as uint)).
            map(|key| key.clone());

        sender.send(result);
    }

    fn set_item(&mut self,  url: Url, name: DOMString, value: DOMString) {
        let origin = self.get_origin_as_string(url);
        if !self.data.contains_key(&origin) {
            self.data.insert(origin.clone(), TreeMap::new());
        }
        self.data.get_mut(&origin).unwrap().insert(name, value);
    }

    fn get_item(&self, sender: Sender<Option<DOMString>>, url: Url, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        let result = self.data.get(&origin)
            .and_then(|entry| entry.get(&name))
            .map(|value| value.to_string());

        sender.send(result);
    }

    fn remove_item(&mut self, url: Url, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        match self.data.get_mut(&origin) {
            Some(origin_data) => {
                origin_data.remove(&name);
            }
            None => {}
        }
    }

    fn clear(&mut self, url: Url) {
        let origin = self.get_origin_as_string(url);
        match self.data.get_mut(&origin) {
            Some(origin_data) => origin_data.clear(),
            None => {}
        }
    }

    fn get_origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_string();
        origin.push_str(url.scheme.as_slice());
        origin.push_str("://");
        url.domain().map(|domain| origin.push_str(domain.as_slice()));
        url.port().map(|port| {
            origin.push_str(":");
            origin.push_str(port.to_string().as_slice());
        });
        origin.push_str("/");
        origin
    }
}
