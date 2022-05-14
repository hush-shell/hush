image:
	@docker build . -t hush-builder

build:
	@docker run --rm -v $(shell pwd)/:/hush hush-builder
