//! Independent wire/projection/default-denial contracts. Managed network tests
//! live beside the private handler so no production destination override exists.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::json;
use substrate_gateway::{
    models::images::*,
    providers::{
        anthropic_compatible::AnthropicCompatibleProvider,
        gemini::{GeminiAuthConfig, GeminiProvider},
        GatewayProvider,
    },
};

#[test]
fn literal_native_request_defaults_and_order() {
    let literal = r#"{"model":"gpt-image-2","prompt":"native α","background":"auto","quality":"auto","size":"auto"}"#;
    let r: ImageGenerationRequest = serde_json::from_str(literal).unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        json!({"model":"gpt-image-2","prompt":"native α","background":"auto","quality":"auto","size":"auto"})
    );
    let r: ImageGenerationRequest =
        serde_json::from_str(r#"{"model":"gpt-image-2","prompt":"native"}"#).unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        json!({"model":"gpt-image-2","prompt":"native"})
    );
    let r:ImageEditRequest=serde_json::from_str(r#"{"model":"gpt-image-2","prompt":"edit","images":[{"image_url":"https://example.test/b"},{"image_url":"https://example.test/a"}]}"#).unwrap();
    assert_eq!(
        r.images
            .iter()
            .map(|i| i.image_url.as_str())
            .collect::<Vec<_>>(),
        vec!["https://example.test/b", "https://example.test/a"]
    );
}

#[test]
fn request_null_unknown_duplicate_wrong_types_are_rejected() {
    for key in ["background", "quality", "size", "n", "prompt", "model"] {
        let mut v = json!({"model":"gpt-image-2","prompt":"sentinel"});
        v[key] = serde_json::Value::Null;
        assert!(serde_json::from_value::<ImageGenerationRequest>(v).is_err());
    }
    for raw in [
        r#"{"model":"gpt-image-2","prompt":"p","mask":"sentinel"}"#,
        r#"{"model":"gpt-image-2","prompt":"p","base_url":"https://sentinel.invalid"}"#,
        r#"{"model":"gpt-image-2","prompt":"p","n":-1}"#,
        r#"{"model":"gpt-image-2","prompt":"p","n":18446744073709551616}"#,
        r#"{"model":"gpt-image-2","prompt":"p","prompt":"duplicate"}"#,
    ] {
        assert!(serde_json::from_str::<ImageGenerationRequest>(raw).is_err());
    }
    for refs in [
        json!(null),
        json!([{"file_id":"file"}]),
        json!([{"image_url":"https://example.test/i","detail":"high"}]),
        json!([{"image_url":null}]),
    ] {
        assert!(serde_json::from_value::<ImageEditRequest>(
            json!({"model":"gpt-image-2","prompt":"p","images":refs})
        )
        .is_err());
    }
}

#[test]
fn named_response_projection_preserves_all_native_nulls_and_omissions() {
    for background in ["transparent", "opaque", "auto"] {
        for quality in ["low", "medium", "high", "auto"] {
            let literal = json!({"created":18446744073709551615u64,"data":[{"b64_json":"source-bytes","generation_id":"gen1"},{"b64_json":"other-bytes","generation_id":null}],"background":background,"quality":quality,"size":"source-size"});
            let r: ImageResponse = serde_json::from_value(literal.clone()).unwrap();
            assert_eq!(serde_json::to_value(r).unwrap(), literal);
        }
    }
    let r:ImageResponse=serde_json::from_str(r#"{"created":0,"data":[{"b64_json":"source","generation_id":null},{"b64_json":"other"}],"background":null,"size":null,"usage":{"private":"sentinel"},"future":[1,2]}"#).unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        json!({"created":0,"data":[{"b64_json":"source","generation_id":null},{"b64_json":"other"}],"background":null,"size":null})
    );
    for literal in [
        json!({"created":null,"data":[]}),
        json!({"created":0,"data":[{"url":"https://example.test/i"}]}),
        json!({"created":0,"data":[{"b64_json":null}]}),
        json!({"created":0,"data":null}),
        json!({"created":0,"data":[],"size":12}),
        json!({"created":0,"data":[],"quality":"future"}),
    ] {
        assert!(serde_json::from_value::<ImageResponse>(literal).is_err());
    }
}

#[test]
fn real_fixture_dimensions_pixels_and_original_bytes() {
    for (bytes, format) in [
        (
            include_bytes!("fixtures/codex_images/red.png").as_slice(),
            image::ImageFormat::Png,
        ),
        (
            include_bytes!("fixtures/codex_images/red.jpg").as_slice(),
            image::ImageFormat::Jpeg,
        ),
        (
            include_bytes!("fixtures/codex_images/red.webp").as_slice(),
            image::ImageFormat::WebP,
        ),
    ] {
        assert_eq!(STANDARD.decode(STANDARD.encode(bytes)).unwrap(), bytes);
        assert_eq!(image::guess_format(bytes).unwrap(), format);
        let decoded = image::load_from_memory_with_format(bytes, format)
            .unwrap()
            .to_rgba8();
        assert_eq!(decoded.dimensions(), (2, 2));
        let expected = if format == image::ImageFormat::Jpeg {
            [254, 0, 0, 255]
        } else {
            [255, 0, 0, 255]
        };
        assert!(decoded.pixels().all(|p| p.0 == expected));
    }
}

#[test]
fn debug_never_displays_transport_labels_or_media() {
    let r: ImageGenerationRequest =
        serde_json::from_str(r#"{"model":"private-sentinel","prompt":"prompt-sentinel"}"#).unwrap();
    let e:ImageEditRequest=serde_json::from_str(r#"{"model":"private-sentinel","prompt":"prompt-sentinel","images":[{"image_url":"https://sentinel.invalid/i"}]}"#).unwrap();
    let response: ImageResponse = serde_json::from_str(
        r#"{"created":0,"data":[{"b64_json":"media-sentinel","generation_id":"id-sentinel"}]}"#,
    )
    .unwrap();
    let mut c = ImageContext::new(tokio::time::Instant::now());
    c.turn_id = Some("label-sentinel".into());
    let result = ImageResult {
        response,
        request_id: Some("request-sentinel".into()),
    };
    for debug in [
        format!("{r:?}"),
        format!("{e:?}"),
        format!("{c:?}"),
        format!("{result:?}"),
    ] {
        assert!(!debug.contains("sentinel"));
    }
}

#[tokio::test]
async fn other_real_providers_deny_both_operations_by_default_without_io() {
    let denied = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", denied.local_addr().unwrap());
    let providers: Vec<Box<dyn GatewayProvider>> = vec![
        Box::new(AnthropicCompatibleProvider::new(
            "a".into(),
            "synthetic".into(),
            url.clone(),
            vec![MODEL.into()],
            None,
            None,
        )),
        Box::new(GeminiProvider::new(
            "g".into(),
            Some(url),
            vec![MODEL.into()],
            Default::default(),
            GeminiAuthConfig {
                api_key: Some("synthetic".into()),
                oauth_provider_id: None,
                token_store: None,
                project_id: None,
                location: None,
            },
        )),
    ];
    for p in providers {
        let r = serde_json::from_str(r#"{"model":"gpt-image-2","prompt":"draw"}"#).unwrap();
        let c = ImageContext::new(tokio::time::Instant::now() + std::time::Duration::from_secs(1));
        assert_eq!(
            p.generate_image(r, c).await.unwrap_err(),
            ImageError::Unsupported
        );
        let r=serde_json::from_str(r#"{"model":"gpt-image-2","prompt":"edit","images":[{"image_url":"https://example.test/i"}]}"#).unwrap();
        let c = ImageContext::new(tokio::time::Instant::now() + std::time::Duration::from_secs(1));
        assert_eq!(
            p.edit_image(r, c).await.unwrap_err(),
            ImageError::Unsupported
        );
    }
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(30), denied.accept())
            .await
            .is_err()
    );
}

#[test]
fn img_r1_valid_extended_alpha_animation_and_lossy_pixels_are_preserved() {
    for (bytes, dimensions, expected) in [
        (
            include_bytes!("fixtures/codex_images/red-lossy.webp").as_slice(),
            (2, 2),
            [255, 1, 0, 255],
        ),
        (
            include_bytes!("fixtures/codex_images/red-extended.webp").as_slice(),
            (2, 2),
            [255, 0, 0, 255],
        ),
        (
            include_bytes!("fixtures/codex_images/red-alpha.webp").as_slice(),
            (2, 2),
            [255, 1, 0, 128],
        ),
        (
            include_bytes!("fixtures/codex_images/red-animation.webp").as_slice(),
            (2, 2),
            [255, 1, 0, 128],
        ),
        (
            include_bytes!("fixtures/codex_images/red-compressed-alpha.webp").as_slice(),
            (16, 16),
            [255, 1, 0, 128],
        ),
        (
            include_bytes!("fixtures/codex_images/red-alpha-lossless.webp").as_slice(),
            (2, 2),
            [255, 0, 0, 128],
        ),
    ] {
        let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::WebP)
            .unwrap()
            .to_rgba8();
        assert_eq!(decoded.dimensions(), dimensions);
        if dimensions == (16, 16) {
            // Lossy compression produces small spatial color variations even
            // for the uniform source. Compare every pixel to an independent
            // ImageMagick decode, not an assumed uniform color or this decoder.
            assert_eq!(
                decoded.as_raw().as_slice(),
                include_bytes!("fixtures/codex_images/red-compressed-alpha.rgba")
            );
        } else {
            assert!(decoded.pixels().all(|p| p.0 == expected));
        }
        let literal = json!({"prompt":"edit","model":"gpt-image-2","images":[{"image_url":format!("data:image/webp;base64,{}",STANDARD.encode(bytes))}]});
        let request: ImageEditRequest = serde_json::from_value(literal.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), literal);
    }
}
