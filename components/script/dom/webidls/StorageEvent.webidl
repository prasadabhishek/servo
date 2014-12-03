/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/webstorage.html#webstorage
 *
 */

[Constructor(DOMString type, optional StorageEventInit eventInitDict)]
interface StorageEvent : Event {
  readonly attribute DOMString? key;
  readonly attribute DOMString? oldValue;
  readonly attribute DOMString? newValue;
  readonly attribute DOMString url;
  readonly attribute Storage? storageArea;
};

dictionary StorageEventInit : EventInit {
  DOMString? key;
  DOMString? oldValue;
  DOMString? newValue;
  DOMString url;
  Storage? storageArea;
};


