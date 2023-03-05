//! S3 plugin for dbgen

use crate::{
    cli::{Env, FileInfo},
    eval::State,
};

use async_trait::async_trait;
use chrono::NaiveDateTime;
use s3_server::dto::{
    Bucket, CompleteMultipartUploadError, CompleteMultipartUploadOutput, CompleteMultipartUploadRequest,
    CopyObjectError, CopyObjectOutput, CopyObjectRequest, CreateBucketError, CreateBucketOutput, CreateBucketRequest,
    CreateMultipartUploadError, CreateMultipartUploadOutput, CreateMultipartUploadRequest, DeleteBucketError,
    DeleteBucketOutput, DeleteBucketRequest, DeleteObjectError, DeleteObjectOutput, DeleteObjectRequest,
    DeleteObjectsError, DeleteObjectsOutput, DeleteObjectsRequest, GetBucketLocationError, GetBucketLocationOutput,
    GetBucketLocationRequest, GetObjectError, GetObjectOutput, GetObjectRequest, HeadBucketError, HeadBucketOutput,
    HeadBucketRequest, HeadObjectError, HeadObjectOutput, HeadObjectRequest, ListBucketsError, ListBucketsOutput,
    ListBucketsRequest, ListObjectsError, ListObjectsOutput, ListObjectsRequest, ListObjectsV2Error,
    ListObjectsV2Output, ListObjectsV2Request, Object as S3Object, PutObjectError, PutObjectOutput, PutObjectRequest,
    UploadPartError, UploadPartOutput, UploadPartRequest,
};
use s3_server::errors::{S3Error, S3ErrorCode, S3StorageResult};
use s3_server::S3Storage;
// use core::slice;

/// A storage implementation for S3.
pub struct Storage {
    env: Env,
    objects: Vec<Object>,
    bucket_name: String,
    current_timestamp: NaiveDateTime,
}

impl Storage {
    /// Create a new storage.
    pub fn new(env: Env, objects: Vec<Object>, current_timestamp: NaiveDateTime) -> Self {
        let bucket_name = env.bucket_name.as_ref().unwrap().clone();
        Self {
            env,
            objects,
            bucket_name,
            current_timestamp,
        }
    }
}

/// A object in storage.
#[derive(Debug, Clone)]
pub struct Object {
    /// The name of the object.
    pub name: String,
    /// The size of the object.
    pub size: usize,
    /// The content of the object.
    pub content: ObjectContent,
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Object {}

/// The content of an object.
#[derive(Clone, Debug)]
pub enum ObjectContent {
    /// A schema object. The string is the schema content.
    Schema(String),
    /// A data object. The content needs to be generated dynamically.
    Data((FileInfo, State)),
}

/// Create a `S3Error` with code and message
macro_rules! code_error {
    ($code:ident, $msg:expr $(, $source:expr)?) => {
        code_error!(code = S3ErrorCode::$code, $msg $(, $source)?)
    };
    (code = $code:expr, $msg:expr $(, $source:expr)?) => {{
        let code = $code;
        let err = S3Error::from_code(code).message($msg);
        $(let err = err.source($source);)?
        let err = err.finish();
        err
    }};
}

/// Create a `NotSupported` error
macro_rules! not_supported {
    ($msg:expr) => {{
        code_error!(NotSupported, $msg)
    }};
}

#[async_trait]
impl S3Storage for Storage {
    async fn complete_multipart_upload(
        &self,
        _: CompleteMultipartUploadRequest,
    ) -> S3StorageResult<CompleteMultipartUploadOutput, CompleteMultipartUploadError> {
        Err(not_supported!("CompleteMultipartUploadRequest").into())
    }

    async fn copy_object(&self, _: CopyObjectRequest) -> S3StorageResult<CopyObjectOutput, CopyObjectError> {
        Err(not_supported!("CopyObjectRequest").into())
    }

    async fn create_multipart_upload(
        &self,
        _: CreateMultipartUploadRequest,
    ) -> S3StorageResult<CreateMultipartUploadOutput, CreateMultipartUploadError> {
        Err(not_supported!("CreateMultipartUploadRequest").into())
    }

    async fn create_bucket(&self, _: CreateBucketRequest) -> S3StorageResult<CreateBucketOutput, CreateBucketError> {
        Err(not_supported!("CreateBucketRequest").into())
    }

    async fn delete_bucket(&self, _: DeleteBucketRequest) -> S3StorageResult<DeleteBucketOutput, DeleteBucketError> {
        Err(not_supported!("DeleteBucketRequest").into())
    }

    async fn delete_object(&self, _: DeleteObjectRequest) -> S3StorageResult<DeleteObjectOutput, DeleteObjectError> {
        Err(not_supported!("DeleteObjectRequest").into())
    }

    async fn delete_objects(
        &self,
        _: DeleteObjectsRequest,
    ) -> S3StorageResult<DeleteObjectsOutput, DeleteObjectsError> {
        Err(not_supported!("DeleteObjectsRequest").into())
    }

    async fn get_bucket_location(
        &self,
        _: GetBucketLocationRequest,
    ) -> S3StorageResult<GetBucketLocationOutput, GetBucketLocationError> {
        Err(not_supported!("GetBucketLocationRequest").into())
    }

    async fn get_object(&self, input: GetObjectRequest) -> S3StorageResult<GetObjectOutput, GetObjectError> {
        if input.bucket != self.bucket_name {
            return Err(code_error!(NoSuchBucket, "The specified bucket does not exist.").into());
        }
        let idx = self
            .objects
            .binary_search_by(|object| object.name.cmp(&input.key))
            .map_err(|_| code_error!(NoSuchKey, "The specified key does not exist."))?;
        let _obj = &self.objects[idx];
        Err(not_supported!("GetObjectRequest").into())
    }

    async fn head_bucket(&self, input: HeadBucketRequest) -> S3StorageResult<HeadBucketOutput, HeadBucketError> {
        if input.bucket == self.bucket_name {
            Ok(HeadBucketOutput)
        } else {
            Err(code_error!(NoSuchBucket, "The specified bucket does not exist.").into())
        }
    }

    async fn head_object(&self, input: HeadObjectRequest) -> S3StorageResult<HeadObjectOutput, HeadObjectError> {
        if input.bucket != self.bucket_name {
            return Err(code_error!(NoSuchBucket, "The specified bucket does not exist.").into());
        }
        let idx = self
            .objects
            .binary_search_by(|object| object.name.cmp(&input.key))
            .map_err(|_| code_error!(NoSuchKey, "The specified key does not exist."))?;
        let obj = &self.objects[idx];

        let last_modified = self.current_timestamp.format("%a, %d %b %Y %T GMT").to_string();
        let output: HeadObjectOutput = HeadObjectOutput {
            content_length: Some(obj.size as i64),
            content_type: Some("application/octet-stream".to_owned()),
            last_modified: Some(last_modified),
            ..HeadObjectOutput::default()
        };
        Ok(output)
    }

    async fn list_buckets(&self, _: ListBucketsRequest) -> S3StorageResult<ListBucketsOutput, ListBucketsError> {
        let bucket = Bucket {
            name: self.env.bucket_name.clone(),
            creation_date: Some(self.current_timestamp.format("%Y-%m-%dT%H:%M:%S.000Z").to_string()),
        };
        let output = ListBucketsOutput {
            buckets: Some(vec![bucket]),
            owner: None,
        };
        Ok(output)
    }

    async fn list_objects(&self, input: ListObjectsRequest) -> S3StorageResult<ListObjectsOutput, ListObjectsError> {
        if input.bucket != self.bucket_name {
            return Err(code_error!(NoSuchBucket, "The specified bucket does not exist.").into());
        }

        let mut s3_objects: Vec<S3Object> = Vec::new();
        let mut next_marker = None;
        for obj in &self.objects {
            if let Some(marker) = &input.marker {
                if &obj.name <= marker {
                    continue;
                }
            }
            if let Some(prefix) = &input.prefix {
                if !obj.name.starts_with(prefix) {
                    continue;
                }
            }
            let s3_object = S3Object {
                key: Some(obj.name.clone()),
                last_modified: Some(self.current_timestamp.format("%Y-%m-%dT%H:%M:%S.000Z").to_string()),
                e_tag: None,
                size: Some(obj.size as i64),
                storage_class: Some("STANDARD".to_string()),
                owner: None,
            };
            if let Some(max_keys) = &input.max_keys {
                if s3_objects.len() == *max_keys as usize {
                    if let Some(last) = s3_objects.last() {
                        next_marker = Some(last.key.clone().unwrap());
                    }
                    break;
                }
            }
            s3_objects.push(s3_object);
        }

        let output = ListObjectsOutput {
            name: Some(self.bucket_name.clone()),
            next_marker,
            prefix: input.prefix,
            marker: input.marker,
            max_keys: input.max_keys,
            is_truncated: Some(false),
            contents: Some(s3_objects),
            common_prefixes: None,
            delimiter: input.delimiter,
            encoding_type: input.encoding_type,
        };
        Ok(output)
    }

    async fn list_objects_v2(
        &self,
        input: ListObjectsV2Request,
    ) -> S3StorageResult<ListObjectsV2Output, ListObjectsV2Error> {
        if input.bucket != self.bucket_name {
            return Err(code_error!(NoSuchBucket, "The specified bucket does not exist.").into());
        }

        let mut s3_objects = Vec::new();
        for obj in &self.objects {
            if let Some(start_after) = &input.start_after {
                if &obj.name <= start_after {
                    continue;
                }
            }
            if let Some(prefix) = &input.prefix {
                if !obj.name.starts_with(prefix) {
                    continue;
                }
            }
            let s3_object = S3Object {
                key: Some(obj.name.clone()),
                last_modified: Some(self.current_timestamp.format("%Y-%m-%dT%H:%M:%S.000Z").to_string()),
                e_tag: None,
                size: Some(obj.size as i64),
                storage_class: Some("STANDARD".to_string()),
                owner: None,
            };
            s3_objects.push(s3_object);
            if let Some(max_keys) = &input.max_keys {
                if s3_objects.len() >= *max_keys as usize {
                    break;
                }
            }
        }
        let key_count = Some(s3_objects.len() as i64);
        let output = ListObjectsV2Output {
            name: Some(self.bucket_name.clone()),
            next_continuation_token: None,
            prefix: input.prefix,
            continuation_token: input.continuation_token,
            max_keys: input.max_keys,
            is_truncated: Some(false),
            contents: Some(s3_objects),
            common_prefixes: None,
            delimiter: input.delimiter,
            encoding_type: input.encoding_type,
            key_count,
            start_after: input.start_after,
        };
        Ok(output)
    }

    async fn put_object(&self, _: PutObjectRequest) -> S3StorageResult<PutObjectOutput, PutObjectError> {
        Err(not_supported!("PutObjectRequest").into())
    }

    async fn upload_part(&self, _: UploadPartRequest) -> S3StorageResult<UploadPartOutput, UploadPartError> {
        Err(not_supported!("UploadPartRequest").into())
    }
}
