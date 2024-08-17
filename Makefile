# 定义目标平台
TARGETS = aarch64-apple-darwin

# 默认目标
.PHONY: all build multi-target-build reset clean run
all: clean multi-target-build reset run
# 子项目路径
SUBPROJECTS = plugins/example_plugin

# 单目标构建任务
build:
	@for project in $(SUBPROJECTS); do \
		(cd $$project && cargo build --release --target=$(TARGET)); \
	done

# 压缩插件任务
zip_plugin:
	@for project in $(SUBPROJECTS); do \
		(cd $$project && mkdir -p ../../dist; \
			cp config.json ../../target/$(TARGET)/release/libexample_plugin.* ../../dist/); \
			(cd ./dist && zip -r example.plugin libexample_plugin.* config.json);\
	done

# 多目标构建任务
multi-target-build:
	@for target in $(TARGETS); do \
		$(MAKE) build TARGET=$$target; \
		$(MAKE) zip_plugin TARGET=$$target; \
	done
reset:
	@rm -rf dist/libexample_plugin.* dist/config.json
clean:
	@cargo clean
	@rm -rf dist
run:
	@cargo run