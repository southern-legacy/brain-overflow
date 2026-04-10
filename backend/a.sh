until mc alias set brain-overflow http://minio:9000 minioadmin minioadmin 2>/dev/null; do
    echo 'Waiting for MinIO to be ready...';
    sleep 2;
done &&
echo 'Creating bucket brain-overflow if not exists...' &&
    mc mb brainoverflow/brain-overflow --ignore-existing &&
echo 'Bucket brain-overflow created or already exists' &&

echo 'Creating webhook of brain-overflow' &&
    mc admin config set brain-overflow notify_webhook:primary \
        endpoint='http://brain-overflow:10086/s3/webhook' &&
    mc admin service restart brain-overflow &&
    mc event add brain-overflow/brain-overflow \
        arn:minio:sqs::primary:webhook --event put,delete &&
    mc event ls brain-overflow/brain-overflow \
        arn:minio:sqs::primary:webhook &&
echo 'Webhook set to http://brain-overflow:10086/s3/webhook, listening put and delete'
