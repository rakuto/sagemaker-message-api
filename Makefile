REGISTRY := 825062525148.dkr.ecr.us-west-2.amazonaws.com
IMAGE := msgapi
TAG ?= latest
REGION ?= us-west-2

ecr-login:
	aws ecr get-login-password --region ${REGION} | docker login --username AWS --password-stdin ${REGISTRY}:
.PHONY: ecr-login

build-image:
	@docker buildx build \
		--platform linux/amd64 \
		-t ${REGISTRY}/${IMAGE}:${TAG} \
		.
.PHONY: build-image

push-image: build-image
	@docker buildx build \
		--platform linux/amd64 \
		-t ${REGISTRY}/${IMAGE}:${TAG} \
		--push \
		.
.PHONY: push-image
