use std::str::FromStr;

use async_compression::tokio::write::BrotliEncoder;
use async_compression::tokio::write::GzipEncoder;
use async_compression::tokio::write::DeflateEncoder;
use tokio::io::AsyncWriteExt;
use strum::EnumString;
use strum::Display;

#[derive(Debug, PartialEq, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum EncodeType { 
    Deflate,
    Gzip,
    #[strum(serialize = "br")]
    Brotli,
}


pub async fn compress_bytes(buf: &[u8], tp: EncodeType ) -> Vec<u8> {
    let ilen = buf.len();
    let mut output = Vec::new();

    match tp {
        EncodeType::Brotli => {
            let mut enc = BrotliEncoder::new(&mut output);
            enc.write_all(buf).await.unwrap();
            enc.flush().await.unwrap();
            enc.shutdown().await.unwrap();
        }
        EncodeType::Gzip => {
            let mut enc = GzipEncoder::new(&mut output);
            enc.write_all(buf).await.unwrap();
            enc.flush().await.unwrap();
            enc.shutdown().await.unwrap();
        }
        EncodeType::Deflate => {
            let mut enc = DeflateEncoder::new(&mut output);
            enc.write_all(buf).await.unwrap();
            enc.flush().await.unwrap();
            enc.shutdown().await.unwrap();
        }
    };

    let olen = output.len();
    println!("{ilen} {olen}");
    output
}


#[cfg(test)]
mod test {
    use super::*;
    use async_compression::tokio::write::BrotliDecoder;
    use async_compression::tokio::write::GzipDecoder;
    use async_compression::tokio::write::DeflateDecoder;
    

    #[test]
    fn test_encode_type() {
        assert_eq!(EncodeType::Brotli, EncodeType::from_str("Br").unwrap());
    }

    #[test]
    #[should_panic]
    fn test_encode_type2() {
        assert_eq!(EncodeType::Brotli, EncodeType::from_str("Brotli").unwrap());
    }

    pub async fn decompress_bytes(buf: &[u8], tp: EncodeType ) -> Vec<u8> {
        let ilen = buf.len();
        let mut output = Vec::new();
    
        match tp {
            EncodeType::Brotli => {
                let mut enc = BrotliDecoder::new(&mut output);
                enc.write_all(buf).await.unwrap();
                enc.flush().await.unwrap();
                enc.shutdown().await.unwrap();
            }
            EncodeType::Gzip => {
                let mut enc = GzipDecoder::new(&mut output);
                enc.write_all(buf).await.unwrap();
                enc.flush().await.unwrap();
                enc.shutdown().await.unwrap();
            }
            EncodeType::Deflate => {
                let mut enc = DeflateDecoder::new(&mut output);
                enc.write_all(buf).await.unwrap();
                enc.flush().await.unwrap();
                enc.shutdown().await.unwrap();
            }
        };
    
        let olen = output.len();
        println!("{ilen} {olen}");
        output
    }

    #[tokio::test]
    async fn test_compression() {
        use rand::{distributions::Standard, Rng};
        let values: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(100000).collect();
        let compressed = compress_bytes(&values, EncodeType::Brotli).await;
        let recoved = decompress_bytes(&compressed, EncodeType::Brotli).await;
        assert_eq!(values, recoved);

        let compressed = compress_bytes(&values, EncodeType::Gzip).await;
        let recoved = decompress_bytes(&compressed, EncodeType::Gzip).await;
        assert_eq!(values, recoved);

        let compressed = compress_bytes(&values, EncodeType::Deflate).await;
        let recoved = decompress_bytes(&compressed, EncodeType::Deflate).await;
        assert_eq!(values, recoved);
    }

}