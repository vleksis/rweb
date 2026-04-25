use std::str::FromStr;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum MediaType {
    #[default]
    TextPlain,
    TextHtml,
    TextCss,
    ApplicationJavascript,
    ApplicationJson,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageSvgXml,
}

impl FromStr for MediaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text/plain" => Ok(Self::TextPlain),
            "text/html" => Ok(Self::TextHtml),
            "text/css" => Ok(Self::TextCss),
            "application/javascript" => Ok(Self::ApplicationJavascript),
            "application/json" => Ok(Self::ApplicationJson),
            "image/png" => Ok(Self::ImagePng),
            "image/jpeg" => Ok(Self::ImageJpeg),
            "image/gif" => Ok(Self::ImageGif),
            "image/svg+xml" => Ok(Self::ImageSvgXml),
            _ => anyhow::bail!("unsupported media type"),
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextPlain => write!(f, "text/plain"),
            Self::TextHtml => write!(f, "text/html"),
            Self::TextCss => write!(f, "text/css"),
            Self::ApplicationJavascript => write!(f, "application/javascript"),
            Self::ApplicationJson => write!(f, "application/json"),
            Self::ImagePng => write!(f, "image/png"),
            Self::ImageJpeg => write!(f, "image/jpeg"),
            Self::ImageGif => write!(f, "image/gif"),
            Self::ImageSvgXml => write!(f, "image/svg+xml"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_media_types() {
        assert_eq!(
            "text/plain".parse::<MediaType>().unwrap(),
            MediaType::TextPlain
        );
        assert_eq!(
            "text/html".parse::<MediaType>().unwrap(),
            MediaType::TextHtml
        );
        assert_eq!("text/css".parse::<MediaType>().unwrap(), MediaType::TextCss);
        assert_eq!(
            "application/javascript".parse::<MediaType>().unwrap(),
            MediaType::ApplicationJavascript
        );
        assert_eq!(
            "application/json".parse::<MediaType>().unwrap(),
            MediaType::ApplicationJson
        );
        assert_eq!(
            "image/png".parse::<MediaType>().unwrap(),
            MediaType::ImagePng
        );
        assert_eq!(
            "image/jpeg".parse::<MediaType>().unwrap(),
            MediaType::ImageJpeg
        );
        assert_eq!(
            "image/gif".parse::<MediaType>().unwrap(),
            MediaType::ImageGif
        );
        assert_eq!(
            "image/svg+xml".parse::<MediaType>().unwrap(),
            MediaType::ImageSvgXml
        );
    }

    #[test]
    fn rejects_unsupported_media_type() {
        let result = "application/brainrot".parse::<MediaType>();

        assert!(result.is_err());
    }
}
