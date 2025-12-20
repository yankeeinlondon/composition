use lib::parse::parse_document;
use lib::types::{DarkMatterNode, Resource, ResourceSource, ResourceRequirement};
use std::path::PathBuf;

#[test]
fn test_parse_audio_directive_in_document() {
    let markdown = r#"# Test Document

Here is some audio:

::audio ./test.mp3

More content here.
"#;

    let resource = Resource {
        source: ResourceSource::Local(PathBuf::from("test.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let doc = parse_document(markdown, resource).unwrap();

    // Find the Audio node
    let audio_node = doc
        .content
        .iter()
        .find(|node| matches!(node, DarkMatterNode::Audio { .. }));

    assert!(audio_node.is_some(), "Audio node should be parsed");

    if let Some(DarkMatterNode::Audio { source, name }) = audio_node {
        assert_eq!(source, "./test.mp3");
        assert!(name.is_none());
    }
}

#[test]
fn test_parse_audio_directive_with_name_in_document() {
    let markdown = r#"::audio ./podcast.mp3 "Episode 42""#;

    let resource = Resource {
        source: ResourceSource::Local(PathBuf::from("test.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let doc = parse_document(markdown, resource).unwrap();

    // Should have at least one Audio node
    let audio_node = doc
        .content
        .iter()
        .find(|node| matches!(node, DarkMatterNode::Audio { .. }));

    assert!(audio_node.is_some(), "Audio node should be parsed");

    if let Some(DarkMatterNode::Audio { source, name }) = audio_node {
        assert_eq!(source, "./podcast.mp3");
        assert_eq!(name, &Some("Episode 42".to_string()));
    }
}

#[test]
fn test_parse_multiple_audio_directives() {
    let markdown = r#"
::audio ./first.mp3 "First Audio"

Some text between.

::audio ./second.mp3

More text.

::audio ./third.mp3 "Third Audio"
"#;

    let resource = Resource {
        source: ResourceSource::Local(PathBuf::from("test.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let doc = parse_document(markdown, resource).unwrap();

    // Count Audio nodes
    let audio_count = doc
        .content
        .iter()
        .filter(|node| matches!(node, DarkMatterNode::Audio { .. }))
        .count();

    assert_eq!(audio_count, 3, "Should parse all three audio directives");
}

#[test]
fn test_parse_audio_with_quoted_path() {
    let markdown = r#"::audio "./path with spaces.mp3" "My Audio""#;

    let resource = Resource {
        source: ResourceSource::Local(PathBuf::from("test.md")),
        requirement: ResourceRequirement::Required,
        cache_duration: None,
    };

    let doc = parse_document(markdown, resource).unwrap();

    let audio_node = doc
        .content
        .iter()
        .find(|node| matches!(node, DarkMatterNode::Audio { .. }));

    assert!(audio_node.is_some(), "Audio node with quoted path should be parsed");

    if let Some(DarkMatterNode::Audio { source, name }) = audio_node {
        assert_eq!(source, "./path with spaces.mp3");
        assert_eq!(name, &Some("My Audio".to_string()));
    }
}

#[test]
fn test_html_renderer_rejects_unprocessed_audio() {
    use lib::render::to_html;

    let nodes = vec![DarkMatterNode::Audio {
        source: "./test.mp3".to_string(),
        name: None,
    }];

    let result = to_html(&nodes);

    assert!(result.is_err(), "to_html should reject unprocessed Audio nodes");

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("Audio directives must be processed"),
            "Error should indicate Audio needs processing"
        );
    }
}
