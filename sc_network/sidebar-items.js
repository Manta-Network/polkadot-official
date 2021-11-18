initSidebarItems({"enum":[["DhtEvent","Events generated by DHT as a response to get_value and put_value requests."],["Event","Type for events generated by networking layer."],["IfDisconnected","When sending a request, what to do on a disconnected recipient."],["ObservedRole","Role that the peer sent to us during the handshake, with the addition of what our local node knows about that peer."],["OutboundFailure","Possible failures occurring in the context of sending an outbound request and receiving the response."],["RequestFailure","Error in a request."],["SyncState","Reported sync state."],["WarpSyncPhase","Reported warp sync phase."]],"mod":[["bitswap","Bitswap server for substrate."],["block_request_handler","Helper for handling (i.e. answering) block requests from a remote peer via the [`crate::request_responses::RequestResponsesBehaviour`]."],["config","Configuration of the networking layer."],["error","Substrate network possible errors."],["light_client_requests","Helpers for outgoing and incoming light client requests."],["multiaddr",""],["network_state","Information about the networking, for diagnostic purposes."],["state_request_handler","Helper for handling (i.e. answering) state requests from a remote peer via the [`crate::request_responses::RequestResponsesBehaviour`]."],["transactions","Transactions handling to plug on top of the network service."],["warp_request_handler","Helper for handling (i.e. answering) grandpa warp sync requests from a remote peer."]],"struct":[["Multiaddr","Representation of a Multiaddr."],["NetworkService","Substrate network service. Handles network IO and manages connectivity."],["NetworkStatus","Overview status of the network."],["NetworkWorker","Main network worker. Must be polled in order for the network to advance."],["NotificationSender","A `NotificationSender` allows for sending notifications to a peer with a chosen protocol."],["NotificationSenderReady","Reserved slot in the notifications buffer, ready to accept data."],["PeerId","Identifier of a peer of the network."],["PeerInfo","Info about a peer’s known state."],["ReputationChange","Description of a reputation adjustment for a node."],["StateDownloadProgress","Reported state download progress."],["WarpSyncProgress","Reported warp sync progress."]],"trait":[["ExHashT","Minimum Requirements for a Hash within Networking"],["NetworkStateInfo","Trait for providing information about the local network state"]]});