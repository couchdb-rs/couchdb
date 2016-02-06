//! The `action` module defines action types.
//!
//! An action is an HTTP method paired with a CouchDB server resource—e.g., `GET
//! /db` and `PUT /db/doc`.
//!
//! Each action has a corresponding type—e.g.,
//! [`GetDatabase`](struct.GetDatabase.html) and
//! [`PutDocument`](struct.PutDocument.html)—used for performing that action.
//!
//! Applications construct actions by calling the appropriate `Client`
//! method—e.g., [`get_database`](../struct.Client.html#method.get_database) and
//! [`put_document`](../struct.Client.html#method.put_document).
//!
//! ## CouchDB API coverage
//!
//! This table shows in detail what parts of the CouchDB API this crate
//! supports.
//!
//! <table>
//!
//!  <thead>
//!   <tr>
//!    <th>URI path</th>
//!    <th>Method</th>
//!    <th>Action type</th>
//!    <th>Header or query parameter</th>
//!    <th align="center">Supported?</th>
//!   </tr>
//!  </thead>
//!
//!  <tbody>
//!   <tr>
//!    <td><code>/</code></td>
//!    <td>GET</td>
//!    <td><a href="struct.GetRoot.html"><code>GetRoot</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_active_tasks</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_all_dbs</code></td>
//!    <td>GET</td>
//!    <td><a href="struct.GetAllDatabases.html"><code>GetAllDatabases</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_db_updates</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_log</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_replicate</td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_restart</td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_stats</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_utils/</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_uuids/</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/favicon.ico/</td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td colspan="4">Basic Authentication</td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="3"><code>/_session</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_config</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/_config/section</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="3"><code>/_config/section/key</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="7"><code>/db</code></td>
//!    <td>HEAD</td>
//!    <td><a href="struct.HeadDatabase.html"><code>HeadDatabase</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td>GET</td>
//!    <td><a href="struct.GetDatabase.html"><code>GetDatabase</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td><a href="struct.PutDatabase.html"><code>PutDatabase</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td><a href="struct.DeleteDatabase.html"><code>DeleteDatabase</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="3">POST</td>
//!    <td rowspan="3"><a href="struct.PostDatabase.html"><code>PostDatabase</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>X-Couch-Full-Commit</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?batch</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_all_docs</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_bulk_docs</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="18"><code>/db/_changes</code></td>
//!    <td rowspan="18">GET</td>
//!    <td rowspan="18"><a href="struct.GetChanges.html"><code>GetChanges</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   <tr>
//!
//!   <tr>
//!    <td><code>Last-Event-Id</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?doc_ids</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?conflicts</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?descending</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?feed</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!   <tr>
//!    <td><code>?filter</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?heartbeat</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!   <tr>
//!    <td><code>?include_docs</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?attachments</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?att_encoding_info</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?last-event-id</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?limit</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?since</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!   <tr>
//!    <td><code>?style</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!   <tr>
//!    <td><code>?timeout</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!   <tr>
//!    <td><code>?view</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_compact</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_compact/design-doc</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_ensure_full_commit</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_view_cleanup</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_security</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_temp_view</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_purge</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_missing_revs</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_revs_diff</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_revs_limit</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="26">
//!     <ul>
//!      <li><code>/db/doc</code></li>
//!      <li><code>/db/_design/design-doc</code></li>
//!      <li><code>/db/_local/id</code></li>
//!     </ul>
//!    </td>
//!    <td rowspan="2">HEAD</td>
//!    <td rowspan="2"><a href="struct.HeadDocument.html"><code>HeadDocument</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>If-None-Match</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="14">GET</td>
//!    <td rowspan="14"><a href="struct.GetDocument.html"><code>GetDocument</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>If-None-Match</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?attachments</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?att_encoding_info</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?atts_since</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?conflicts</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?deleted_conflicts</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?latest</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?local_seq</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?meta</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?open_revs</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?rev</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?revs</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?revs_info</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="4">PUT</td>
//!    <td rowspan="4"><a href="struct.PutDocument.html"><code>PutDocument</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>If-Match</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>X-Couch-Full-Commit</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?batch</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="5">DELETE</td>
//!    <td rowspan="5"><a href="struct.DeleteDocument.html"><code>DeleteDocument</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>If-Match</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>X-Couch-Full-Commit</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?rev</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?batch</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>COPY</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="4">
//!     <ul>
//!      <li><code>/db/doc/attachment</code></li>
//!      <li><code>/db/doc/_design/design-doc/attachment</code></li>
//!     </ul>
//!    </td>
//!    <td>HEAD</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_design/design-doc/_info</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="20"><code>/db/_design/design-doc/_view/view-name</code></td>
//!    <td rowspan="19">GET</td>
//!    <td rowspan="19"><a href="struct.GetView.html"><code>GetView</code></a></td>
//!    <td/>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?conflicts</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?descending</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?endkey</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?endkey_doc</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?group</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?group_level</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?include_docs</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?attachments</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?att_encoding_info</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?inclusive_end</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?key</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?limit</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?reduce</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?skip</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?stale</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?startkey</code></td>
//!    <td align="center" style="color:green;">✓</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?startkey_docid</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>?update_seq</code></td>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_design/design-doc/_show/show-name</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_design/design-doc/_show/show-name/doc-id</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="2"><code>/db/_design/design-doc/_list/list-name/view-name</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td
//!    rowspan="2"><code>/db/_design/design-doc/_list/list-name/other-ddoc/view-name</code></td>
//!    <td>GET</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_design/design-doc/_update/update-name</code></td>
//!    <td>POST</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_design/design-doc/_update/update-name/doc-id</code></td>
//!    <td>PUT</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!   <tr>
//!    <td><code>/db/_design/design-doc/_rewrite/path</code></td>
//!    <td>ANY</td>
//!    <td/>
//!    <td/>
//!    <td align="center" style="color:darkred;">❌</td>
//!   </tr>
//!
//!  </tbody>
//! </table>

macro_rules! impl_action_public_methods {
    ($action_output:ty) => {
        /// Sends the action request and waits for the response.
        pub fn run(self) -> Result<$action_output, Error> {
            action::run_action(self)
        }
    }
}

macro_rules! make_couchdb_error {
    ($error_variant:ident, $response:expr) => {
        Error::$error_variant(Some(try!($response.decode_json_all::<ErrorResponse>())))
    }
}

#[macro_use]
mod test_macro;

mod delete_database;
mod delete_document;
mod get_all_databases;
mod get_changes;
mod get_database;
mod get_document;
mod get_root;
mod get_view;
mod head_database;
mod head_document;
mod post_database;
mod put_database;
mod put_document;

pub use self::delete_database::DeleteDatabase;
pub use self::delete_document::DeleteDocument;
pub use self::get_all_databases::GetAllDatabases;
pub use self::get_changes::{GetChanges, ChangesEvent, ChangesSince};
pub use self::get_database::GetDatabase;
pub use self::get_document::GetDocument;
pub use self::get_root::GetRoot;
pub use self::get_view::GetView;
pub use self::head_database::HeadDatabase;
pub use self::head_document::HeadDocument;
pub use self::post_database::{PostDatabase, PostToDatabase};
pub use self::put_database::PutDatabase;
pub use self::put_document::PutDocument;

use hyper;
use serde;
use std;
use std::io::prelude::*;

#[cfg(test)]
use serde_json;

use Error;
use Revision;
use error::TransportKind;

// The Action trait abstracts the machinery for executing CouchDB actions. Types
// implementing the Action trait define only how they construct requests and
// process responses. This separates the action logic from the responsibility of
// sending the request and receiving its response.
trait Action: Sized {
    type Output;
		type State;
    fn make_request(self) -> Result<(Request, Self::State), Error>;
    fn take_response<R>(response: R, state: Self::State) -> Result<Self::Output, Error>
        where R: Response;
}

fn run_action<A>(action: A) -> Result<A::Output, Error>
    where A: Action
{
    let (action_request, action_state) = try!(action.make_request());

    let action_response = {
        use std::io::Write;
        let mut hyper_request = try!(hyper::client::Request::new(action_request.method,
                                                                 action_request.uri)
                                         .map_err(|e| Error::Transport(TransportKind::Hyper(e))));
        *hyper_request.headers_mut() = action_request.headers;
        let mut request_stream = try!(hyper_request.start()
                                                   .map_err(|e| {
                                                       Error::Transport(TransportKind::Hyper(e))
                                                   }));
        try!(request_stream.write_all(&action_request.body)
                           .map_err(|e| Error::Transport(TransportKind::Io(e))));
        let hyper_response = try!(request_stream.send().map_err(|e| {
            Error::Transport(TransportKind::Hyper(e))
        }));
        HyperResponse::new(hyper_response)
    };

    A::take_response(action_response, action_state)
}

struct Request {
    method: hyper::method::Method,
    uri: hyper::Url,
    headers: hyper::header::Headers,
    body: Vec<u8>,
}

impl Request {
    pub fn new(method: hyper::method::Method, uri: hyper::Url) -> Self {
        Request {
            method: method,
            uri: uri,
            headers: hyper::header::Headers::new(),
            body: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn method(&self) -> &hyper::method::Method {
        &self.method
    }

    #[cfg(test)]
    pub fn uri(&self) -> &hyper::Url {
        &self.uri
    }

    #[cfg(test)]
    pub fn headers(&self) -> &hyper::header::Headers {
        &self.headers
    }

    pub fn set_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn set_accept_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            vec![hyper::header::qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]
        };
        self.headers.set(hyper::header::Accept(qitems));
        self
    }

    pub fn set_content_type_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            Mime(TopLevel::Application, SubLevel::Json, vec![])
        };
        self.headers.set(hyper::header::ContentType(qitems));
        self
    }

    pub fn set_if_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.headers.set(hyper::header::IfMatch::Items(etags));
                self
            }
        }
    }

    pub fn set_if_none_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.headers.set(hyper::header::IfNoneMatch::Items(etags));
                self
            }
        }
    }
}

trait Response {

    // Returns the HTTP status code.
    fn status(&self) -> hyper::status::StatusCode {
        unimplemented!();
    }

    // Returns an error if and only if the response does not have a Content-Type
    // header equivalent to 'application/json'.
    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        unimplemented!();
    }

    // Decodes the entire response body as JSON.
    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        unimplemented!();
    }

    // Decodes the next line of the response body as JSON. Returns None if and
    // only if EOF is reached without reading a line.
    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        unimplemented!();
    }

    // Returns an error if and only if the response body has non-whitespace
    // remaining.
    fn no_more_content(&mut self) -> Result<(), Error> {
        unimplemented!();
    }
}

mod json {
    use serde;
    use serde_json;
    use std;

    use Error;
    use error::{DecodeErrorKind, TransportKind};

    pub fn decode_json_all<R, T>(reader: &mut R) -> Result<T, Error>
        where R: std::io::Read,
              T: serde::Deserialize
    {
        let reader = reader.by_ref();
        serde_json::from_reader(reader).map_err(|e| {
            match e {
                serde_json::Error::IoError(e) => Error::Transport(TransportKind::Io(e)),
                _ => Error::Decode(DecodeErrorKind::Serde { cause: e }),
            }
        })
    }

    pub fn decode_json_line<R, T>(reader: &mut R) -> Result<T, Error>
        where R: std::io::BufRead,
              T: serde::Deserialize
    {
        let mut s = String::new();
        try!(reader.read_line(&mut s)
                   .map_err(|e| Error::Transport(TransportKind::Io(e))));
        serde_json::from_str::<T>(&s)
            .map_err(|e| Error::Decode(DecodeErrorKind::Serde { cause: e }))
    }

    pub fn no_more_content<R>(reader: &mut R) -> Result<(), Error>
        where R: std::io::Read
    {
        let mut s = String::new();
        try!(reader.read_to_string(&mut s)
                   .map_err(|e| Error::Transport(TransportKind::Io(e))));
        for c in s.chars() {
            match c {
                '\r' | '\n' | ' ' => (),
                _ => {
                    return Err(Error::Decode(DecodeErrorKind::TrailingContent));
                }
            }
        }
        Ok(())
    }
}

struct HyperResponse {
    hyper_response: std::io::BufReader<hyper::client::Response>,
}

impl HyperResponse {
    fn new(hyper_response: hyper::client::Response) -> Self {
        HyperResponse { hyper_response: std::io::BufReader::new(hyper_response) }
    }
}

impl Response for HyperResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.hyper_response.get_ref().status
    }

    // Returns an error if the HTTP response doesn't have a Content-Type of
    // `application/json`.
    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        headers_content_type_must_be_application_json(&self.hyper_response.get_ref().headers)
    }

    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_all(&mut self.hyper_response)
    }

    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_line(&mut self.hyper_response)
    }

    fn no_more_content(&mut self) -> Result<(), Error> {
        json::no_more_content(&mut self.hyper_response)
    }
}

// Mock response encapsulating a typical application/json response.
#[cfg(test)]
struct JsonResponse {
    status_code: hyper::status::StatusCode,
    body: std::io::BufReader<std::io::Cursor<String>>,
}

#[cfg(test)]
impl JsonResponse {
    fn new<T: serde::Serialize>(status_code: hyper::status::StatusCode, body: &T) -> Self {
        let body = serde_json::to_string(&body).unwrap();
        let body = std::io::BufReader::new(std::io::Cursor::new(body));
        JsonResponse {
            status_code: status_code,
            body: body,
        }
    }

    fn new_from_string<S>(status_code: hyper::status::StatusCode, body: S) -> Self
        where S: Into<String>
    {
        JsonResponse {
            status_code: status_code,
            body: std::io::BufReader::new(std::io::Cursor::new(body.into())),
        }
    }
}

#[cfg(test)]
impl Response for JsonResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.status_code
    }

    fn content_type_must_be_application_json(&self) -> Result<(), Error> {
        Ok(())
    }

    fn decode_json_all<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_all(&mut self.body)
    }

    fn decode_json_line<T: serde::Deserialize>(&mut self) -> Result<T, Error> {
        json::decode_json_line(&mut self.body)
    }

    fn no_more_content(&mut self) -> Result<(), Error> {
        json::no_more_content(&mut self.body)
    }
}

// Mock response encapsulating a response with no body.
#[cfg(test)]
struct NoContentResponse {
    status_code: hyper::status::StatusCode,
}

#[cfg(test)]
impl NoContentResponse {
    fn new(status_code: hyper::status::StatusCode) -> Self {
        NoContentResponse { status_code: status_code }
    }
}

#[cfg(test)]
impl Response for NoContentResponse {
    fn status(&self) -> hyper::status::StatusCode {
        self.status_code
    }
}

fn headers_content_type_must_be_application_json(headers: &hyper::header::Headers)
                                                 -> Result<(), Error> {
    use hyper::header::ContentType;
    use hyper::mime::{Mime, TopLevel, SubLevel};
    let c = "application/json";
    match headers.get::<ContentType>() {
        None => Err(Error::NoContentTypeHeader { expected: c }),
        Some(&ContentType(Mime(TopLevel::Application, SubLevel::Json, ref _param))) => Ok(()),
        Some(&ContentType(ref mime)) => {
            Err(Error::UnexpectedContentTypeHeader {
                expected: c,
                got: format!("{}", mime),
            })
        }
    }
}

fn new_revision_etags(rev: &Revision) -> Vec<hyper::header::EntityTag> {
    vec![hyper::header::EntityTag::new(false, rev.to_string())]
}

#[cfg(test)]
mod tests {

    #[test]
    fn headers_content_type_must_be_application_json_ok_with_charset() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("application/json; charset=utf-8".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap();
    }

    #[test]
    fn headers_content_type_must_be_application_json_ok_without_charset() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("application/json".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap();
    }

    #[test]
    fn headers_content_type_must_be_application_json_no_header() {
        use hyper::header::Headers;
        let headers = Headers::new();
        super::headers_content_type_must_be_application_json(&headers).unwrap_err();
    }

    #[test]
    fn headers_content_type_must_be_application_json_wrong_type() {
        use hyper::header::{ContentType, Headers};
        let mut headers = Headers::new();
        headers.set(ContentType("plain/text".parse().unwrap()));
        super::headers_content_type_must_be_application_json(&headers).unwrap_err();
    }
}
