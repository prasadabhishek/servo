/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StorageBinding;
use dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use servo_util::str::DOMString;
use servo_net::storage_task::StorageTask;
use servo_net::storage_task::StorageTaskMsg;
use std::comm::channel;

#[dom_struct]
pub struct Storage {
    reflector_: Reflector,
    global: GlobalField,
}

impl Storage {
    fn new_inherited(global: &GlobalRef) -> Storage {
        Storage {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Storage> {
        reflect_dom_object(box Storage::new_inherited(global), global, StorageBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Storage>> {
        Ok(Storage::new(global))
    }

    fn get_origin_as_string(&self) -> String {

        let global_root = self.global.root();
        let global_ref = global_root.root_ref();
        let url = global_ref.get_url();

        let mut origin = "".to_string();
        origin.push_str(url.scheme.as_slice());
        origin.push_str("://");
        if url.domain() != None {
            origin.push_str(url.domain().unwrap().as_slice());
        }
        origin.push_str("/");
        if url.port() != None {
            origin.push_str(url.port().unwrap().to_string().as_slice());
        }
        return origin;
    }

    fn get_storage_task(&self) -> StorageTask {

        let global_root = self.global.root();
        let global_ref = global_root.root_ref();
        global_ref.storage_task()
    }

}

impl<'a> StorageMethods for JSRef<'a, Storage> {
    fn Length(self) -> u32 {
        let (sender, receiver) = channel();

        let origin = self.get_origin_as_string();
        let storage_task = self.get_storage_task();

        storage_task.send(StorageTaskMsg::Length(sender, origin));
        receiver.recv()
    }

    fn Key(self, index: u32) -> Option<DOMString> {
        let (sender, receiver) = channel();

        let origin = self.get_origin_as_string();
        let storage_task = self.get_storage_task();

        storage_task.send(StorageTaskMsg::Key(sender, origin, index));
        receiver.recv()
    }

    fn GetItem(self, name: DOMString) -> Option<DOMString> {
        let (sender, receiver) = channel();

        let origin = self.get_origin_as_string();
        let storage_task = self.get_storage_task();

        storage_task.send(StorageTaskMsg::GetItem(sender, origin, name));
        receiver.recv()
    }

    fn NamedGetter(self, name: DOMString, found: &mut bool) -> Option<DOMString> {
        let item = self.GetItem(name);
        *found = item.is_some();
        item
    }

    fn SetItem(self, name: DOMString, value: DOMString) {
        //update value only if the given name/value pair does not exist
        let item = self.GetItem(name.clone());
        if !item.is_some() || item.unwrap().as_slice() != value.as_slice() {
            let origin = self.get_origin_as_string();
            let storage_task = self.get_storage_task();

            storage_task.send(StorageTaskMsg::SetItem(origin, name, value));
        }
    }

    fn NamedSetter(self, name: DOMString, value: DOMString) {
        self.SetItem(name, value);
    }

    fn NamedCreator(self, name: DOMString, value: DOMString) {
        self.SetItem(name, value);
    }

    fn RemoveItem(self, name: DOMString) {
        //remove value only if the given name/value pair does not exist
        let item = self.GetItem(name.clone());
        if item.is_some() {
            let origin = self.get_origin_as_string();
            let storage_task = self.get_storage_task();

            storage_task.send(StorageTaskMsg::RemoveItem(origin, name));
        }
    }

    fn NamedDeleter(self, name: DOMString) {
        self.RemoveItem(name);
    }

    fn Clear(self) {
        let origin = self.get_origin_as_string();
        let storage_task = self.get_storage_task();

        storage_task.send(StorageTaskMsg::Clear(origin));
    }
}

impl Reflectable for Storage {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
