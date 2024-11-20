use super::*;

#[test]
fn test_namespace() {
    let mut dz = DagZet::new();

    dz.parse_line("ns hello");

    assert_eq!(dz.namespace, Some("hello".to_string()));
}

#[test]
fn test_graph_remarks() {
    let mut dz = DagZet::new();
    dz.parse_line("ns hello");
    dz.parse_line("gr this is a graph remark");
    dz.parse_line("gr for the node called hello");

    assert_eq!(dz.graph_remarks.len(), 1);
    assert!(dz.graph_remarks.contains_key("hello"));

    let gr = dz.graph_remarks;

    match gr.get("hello") {
        Some(remarks) => {
            assert_eq!(remarks.len(), 2);
            assert_eq!(remarks[0], "this is a graph remark");
            assert_eq!(remarks[1], "for the node called hello");
        }
        None => {
            // Shouldn't happen, since there was a check before this
        }
    };
}
#[test]
fn test_new_node() {
    let mut dz = DagZet::new();
    let caught_no_namespace = match dz.parse_line_with_result("nn hello") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NameSpaceNotSet),
    };
    assert!(caught_no_namespace);

    // catch multiple node declared error
    let mut dz = DagZet::new();

    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");

    assert_eq!(dz.nodes.len(), 1, "Expected nodes.");
    assert_eq!(
        dz.nodelist.len(),
        dz.nodes.len(),
        "nodelist inconsistency: wrong size."
    );

    let caught_duplicate_node = match dz.parse_line_with_result("nn bbb") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NodeAlreadyExists),
    };
    assert!(caught_duplicate_node);
    assert!(dz.nodes.contains_key("aaa/bbb"));

    let node_id = dz.nodes.get("aaa/bbb").unwrap();
    let node_id = *node_id as usize - 1;

    let maps_to_nodelist = dz.nodelist[node_id] == "aaa/bbb";

    assert!(
        maps_to_nodelist,
        "nodelist inconsistency: ID mapping broken"
    );
}

#[test]
fn test_lines() {
    let mut dz = DagZet::new();
    // attempt to parse lines without select a node
    dz.parse_line("ns aaa");

    let caught_missing_node = match dz.parse_line_with_result("ln hello line") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
    };

    assert!(caught_missing_node);

    let mut dz = DagZet::new();
    // attempt to parse lines without select a node
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("ln ccc");
    dz.parse_line("ln another line");

    // Make sure the lines are behaving as expected.
    assert_eq!(dz.lines.len(), 1);
    assert!(dz.nodes.contains_key("aaa/bbb"));

    let node_id = dz.nodes.get("aaa/bbb").unwrap();

    if let Some(ln) = dz.lines.get(node_id) {
        assert_eq!(ln.len(), 2);
        assert_eq!(ln[0], "ccc");
        assert_eq!(ln[1], "another line");
    }
}
#[test]
fn test_connect() {
    let mut dz = DagZet::new();
    dz.parse_line("ns top");
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");

    let result = match dz.parse_line_with_result("co bbb") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NotEnoughArgs),
    };

    assert!(result, "Did not catch NotEnoughArgs error");

    dz.parse_line("co bbb aaa");

    assert_eq!(
        dz.connections.len(),
        1,
        "expected a single connection to be made"
    );

    let c = &dz.connections[0];

    let aaa = "top/aaa";
    let bbb = "top/bbb";

    assert_eq!(&c[0], bbb, "expected top/bbb node in left connection");
    assert_eq!(&c[1], aaa, "expected top/aaa node in right connection");

    // Make sure different namespaces work
    dz.parse_line("ns pot");
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("co bbb aaa");

    let c = &dz.connections[1];
    let aaa = "pot/aaa";
    let bbb = "pot/bbb";

    assert_eq!(&c[0], bbb, "expected pot/bbb node in left connection");
    assert_eq!(&c[1], aaa, "expected pot/aaa node in right connection");

    // make sure repeated connections aren't attempted
    let result = match dz.parse_line_with_result("co bbb aaa") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::AlreadyConnected),
    };

    assert!(result, "Did not catch AlreadyConnected error");
}

#[test]
fn test_connect_shorthands() {
    let mut dz = DagZet::new();
    dz.parse_line("ns top");

    // Make sure shorthand returns an error if a node
    // isn't selected.
    // Note that it doesn't matter if 'bbb' exist or not
    // those checks don't happen until after all the nodes
    // are created.
    let result = match dz.parse_line_with_result("co $ bbb") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
    };
    assert!(result, "Did not catch NodeNotSelected error");

    // Make nodes aaa and bbb, then use shorthand to connect
    // bbb -> aaa
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("co $ aaa");

    assert_eq!(dz.connections.len(), 1, "no connections found");

    let co = &dz.connections[0];
    // Test lefthand shorthand
    assert_eq!(co[0], "top/bbb", "left shorthand does not work");

    // Test righthand shorthand for bbb -> ccc
    dz.parse_line("nn ccc");
    dz.parse_line("co bbb $");

    let co = &dz.connections[1];
    assert_eq!(co[1], "top/ccc", "right shorthand does not work");
}

#[test]
fn test_connection_remarks() {
    let mut dz = DagZet::new();

    let result = match dz.parse_line_with_result("cr no connections have been made yet") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NoConnections),
    };
    assert!(result, "Did not catch NoConnections error");

    dz.parse_line("ns top");
    dz.parse_line("co aaa bbb");
    dz.parse_line("cr this is a remark");

    // make sure connection remark is made
    assert_eq!(
        dz.connection_remarks.len(),
        1,
        "Expected a connection remark to appear."
    );

    // Make sure appending works

    dz.parse_line("cr this is a remark on another line");

    // grab the connection remark, make sure appending works

    let co = dz.connection_remarks.get(&0).unwrap();

    assert_eq!(co.len(), 2, "Expected 2 lines in this remark");

    assert_eq!(co[0], "this is a remark");
    assert_eq!(co[1], "this is a remark on another line");
}

#[test]
fn test_invalid_command() {
    let mut dz = DagZet::new();

    let result = match dz.parse_line_with_result("xx this isn't a real command") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::InvalidCommand),
    };
    assert!(result, "Did not catch InvalidCommand error");
}

#[test]
fn test_unknown_nodes() {
    let mut dz = DagZet::new();

    dz.parse_line("ns top");
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");

    dz.parse_line("co aaa bbb");
    dz.parse_line("co aaa ccc");
    dz.parse_line("co ccc ddd");

    let unknown = dz.check_unknown_nodes();

    assert_eq!(unknown.len(), 2, "Wrong number of expected nodes");

    assert!(unknown.contains("top/ccc"));
    assert!(unknown.contains("top/ddd"));
}

#[test]
fn test_check_for_loops() {
    let mut dz = DagZet::new();

    dz.parse_line("ns top");
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");

    dz.parse_line("co aaa bbb");
    dz.parse_line("co bbb aaa");

    assert_eq!(dz.check_unknown_nodes().len(), 0, "Found unknown nodes");
    let edges = dz.generate_edges();

    assert!(dz.check_for_loops(&edges).is_err(), "Did not catch cycles");
}

#[test]
fn test_comments() {
    let mut dz = DagZet::new();

    let result = dz.parse_line_with_result("zz this is a comment").is_ok();
    assert!(result, "Did not properly ignore comment");
}

#[test]
fn test_node_remarks() {
    let mut dz = DagZet::new();
    // attempt to parse lines without select a node
    dz.parse_line("ns aaa");

    let caught_missing_node = match dz.parse_line_with_result("rm hello remark") {
        Ok(_) => false,
        Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
    };

    assert!(
        caught_missing_node,
        "tried to make a remark on unselected node"
    );

    let mut dz = DagZet::new();
    // attempt to parse lines without select a node
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("rm ccc");
    dz.parse_line("rm another line");

    // Make sure the remarks are behaving as expected.
    assert_eq!(dz.node_remarks.len(), 1, "couldn't find remarks");

    let node_id = dz.nodes.get("aaa/bbb").unwrap();

    if let Some(rm) = dz.lines.get(node_id) {
        assert_eq!(rm.len(), 2);
        assert_eq!(rm[0], "ccc");
        assert_eq!(rm[1], "another line");
    }
}
#[test]
fn test_file_range() {
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("fr foo 1 4");
    let fr = &dz.file_ranges[&dz.curnode.unwrap()];
    assert!(
        fr.start == 1 && fr.end == 4,
        "could not properly handle full file range"
    );

    // make sure file range is in the valid order
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    let result = dz.parse_line_with_result("fr foo 4 1");
    assert!(result.is_err(), "wrong order for line not caught");

    // make sure file range is valid number
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    let result = dz.parse_line_with_result("fr foo one 4");
    assert!(
        result.is_err(),
        "didn't catch invalid numbers for file range"
    );

    // file range with one line
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("fr foo 4");
    let fr = &dz.file_ranges[&dz.curnode.unwrap()];
    assert!(
        fr.start == 4 && fr.end == -1,
        "could not handle file range with one line"
    );

    // file range with no lines (whole file)
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("fr foo");
    let fr = &dz.file_ranges[&dz.curnode.unwrap()];
    assert!(
        fr.start == -1 && fr.end == -1,
        "could not handle file range with one line"
    );

    // shorthand working as expected
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("fr foo 1 4");
    dz.parse_line("nn ccc");
    dz.parse_line("fr $ 3 5");
    let fr = &dz.file_ranges[&dz.curnode.unwrap()];
    assert!(
        fr.start == 3 && fr.end == 5,
        "could not handle shorthand as expected"
    );

    // attempt shorthand without setting file beforehand
    let mut dz = DagZet::new();
    dz.parse_line("ns aaa");
    dz.parse_line("nn bbb");
    let result = dz.parse_line_with_result("fr $ 1 4");
    assert!(
        result.is_err(),
        "shorthand did not fail as it was supposed to"
    );
}

#[test]
fn test_hyperlinks() {
    // Test usual functionality
    let mut dz = DagZet::new();
    dz.parse_line("ns links");
    dz.parse_line("nn internet_archive");
    dz.parse_line("hl http://archive.org");

    assert_eq!(
        dz.hyperlinks.len(),
        1,
        "Expected exactly one entry in hyperlinks"
    );

    let curnode = &dz.curnode.unwrap();

    let hl = &dz.hyperlinks[curnode];

    assert_eq!(hl, "http://archive.org", "wrong hyperlink found");

    // Test hyperlink without node selected
    let mut dz = DagZet::new();
    dz.parse_line("ns links");
    let result = dz.parse_line_with_result("hl http://archive.org");

    assert!(
        result.is_err_and(|x| { matches!(x, ReturnCode::NodeNotSelected) }),
        "Did not catch NodeNotSelected"
    );
}

#[test]
fn test_todo() {
    // make sure default behavior works
    let mut dz = DagZet::new();
    dz.parse_line("ns top");
    dz.parse_line("nn aaa");
    dz.parse_line("td todo item");

    assert_eq!(dz.todos.len(), 1, "Expected TODO item");

    let curnode = &dz.curnode.unwrap();

    let todostr = &dz.todos[curnode];

    assert_eq!(todostr, "todo item", "incorrect TODO item found");

    let mut dz = DagZet::new();
    dz.parse_line("ns top");
    let result = dz.parse_line_with_result("td todo item");

    assert!(
        result.is_err_and(|x| { matches!(x, ReturnCode::NodeNotSelected) }),
        "Did not catch NodeNotSelected"
    );
}

#[test]
fn test_tags() {
    let mut dz = DagZet::new();
    dz.parse_line("ns top");
    let result = dz.parse_line_with_result("tg oops");

    assert!(
        result.is_err_and(|x| { matches!(x, ReturnCode::NodeNotSelected) }),
        "Did not catch NodeNotSelected"
    );

    let mut dz = DagZet::new();

    // Test out conventional tag behavior"
    dz.parse_line("ns seuss");
    dz.parse_line("nn fishes");
    dz.parse_line("tg one two red blue");

    let curnode = &dz.curnode.unwrap();
    assert_eq!(dz.tags.len(), 1, "Expected tags to have an item");

    let tags = &dz.tags[curnode];

    assert_eq!(tags.len(), 4, "not enough tags");

    assert!(tags.contains("one"));
    assert!(tags.contains("two"));
    assert!(tags.contains("red"));
    assert!(tags.contains("blue"));

    // Make sure append behavior works
    dz.parse_line("tg green");
    let tags = &dz.tags[curnode];

    assert_eq!(tags.len(), 5, "expected one more tag to be appended");
    assert!(tags.contains("green"));

    // Make sure tag doesn't appear twice
    dz.parse_line("nn do_not_like");
    let result = dz.parse_line_with_result("tg green_eggs ham green_eggs");

    assert!(result.is_err(), "did not error on duplicate tags");
}

#[test]
fn test_select_node() {
    let mut dz = DagZet::new();

    dz.parse_line("ns top");
    dz.parse_line("nn aaa");
    dz.parse_line("nn bbb");
    dz.parse_line("sn aaa");

    let curnode = dz.curnode.unwrap();

    assert_eq!(
        dz.nodelist[curnode as usize - 1],
        "top/aaa",
        "wrong node selected"
    );

    let result = dz.parse_line_with_result("sn ccc");

    assert!(result.is_err());
}

#[test]
// 2024-07-24 Discovered this issue in dagzet, my topsort
// cycle checker wasn't implemented correctly
fn test_remaining_edges_bug() {
    let mut dz = DagZet::new();
    dz.parse_line("ns top");
    dz.parse_line("nn ink");
    dz.parse_line("co $ scans");
    dz.parse_line("nn text");
    dz.parse_line("nn html");
    dz.parse_line("nn schedule");
    dz.parse_line("nn repo");
    dz.parse_line("co scans $");
    dz.parse_line("co text $");
    dz.parse_line("nn scans");
    let result = dz.check_for_loops(&dz.generate_edges());
    assert!(result.is_ok());
}

// Initial "cx" behavior doesn't need aliases or shorthands
// only full path connections
#[test]
fn test_cx_initial() {
    let mut dz = DagZet::new();
    let result = dz.parse_line_with_result("cx colors/fishes");

    assert!(
        result.is_err_and(|x| { matches!(x, ReturnCode::NotEnoughArgs) }),
        "Did not catch NotEnoughArgs"
    );

    dz.parse_line("cx colors/fishes numbers/fishes");
    dz.parse_line("cr kinds of fishes in dr.seuss");

    assert_eq!(dz.xnodes.len(), 2, "Expected 2 xnodes");
    assert_eq!(dz.connections.len(), 1, "Expected 1 connection");
    assert_eq!(
        dz.connection_remarks.len(),
        1,
        "Expected 1 connection remark"
    );

    let result = dz.check_unknown_nodes();

    assert!(result.is_empty(), "There shouldn't be any unknown nodes");

    let result = dz.parse_line_with_result("cx colors/fishes numbers/fishes");

    assert!(
        result.is_err_and(|x| { matches!(x, ReturnCode::AlreadyConnected) }),
        "Did not catch AlreadyConnected"
    );
}

#[test]
fn test_flashcard() {
    let mut dz = DagZet::new();
    dz.parse_line("ns test");
    dz.parse_line("nn a");
    dz.parse_line("ff front of card");
    dz.parse_line("fb back of card");

    let curnode = &dz.curnode.unwrap();
    assert!(dz.flashcards.contains_key(curnode));

    let card = dz.flashcards.get(curnode).unwrap();
    assert!(card.front.len() == 1);
    assert!(card.front[0] == "front of card");
    assert!(card.back.len() == 1);
    assert!(card.back[0] == "back of card");

    dz.parse_line("nn b");
    dz.parse_line("ff one");
    dz.parse_line("ff two");
    let curnode = &dz.curnode.unwrap();
    let card = dz.flashcards.get(curnode).unwrap();
    assert!(card.front.len() == 2);
    assert!(card.back.is_empty());
}

#[test]
fn test_image() {
    let mut dz = DagZet::new();
    dz.parse_line("ns a");
    dz.parse_line("nn b");
    dz.parse_line("im c.jpg");

    assert_eq!(dz.images.len(), 1);
    let curnode = dz.curnode.unwrap();

    let filename = &dz.images.get(&curnode).unwrap();

    assert_eq!(filename, &"c.jpg");
}

#[test]
fn test_audio() {
    let mut dz = DagZet::new();
    dz.parse_line("ns a");
    dz.parse_line("nn b");
    dz.parse_line("au c.mp3");

    assert_eq!(dz.audio.len(), 1);
    let curnode = dz.curnode.unwrap();

    let filename = &dz.audio.get(&curnode).unwrap();

    assert_eq!(filename, &"c.mp3");
}
