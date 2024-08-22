TARGETS = aarch64-apple-darwin

.PHONY: all build multi-target-build reset clean run
all: clean multi-target-build reset run

# 动态查找 plugins 目录下的所有子项目
SUBPROJECTS = $(shell find plugins -mindepth 1 -maxdepth 1 -type d)

# 单目标构建任务
build:
	@for project in $(SUBPROJECTS); do \
		(cd $$project && cross build --release --target=$(TARGET)); \
	done

# 压缩插件任务
zip_plugin:
	@for project in $(SUBPROJECTS); do \
		(cd $$project && mkdir -p ../../dist; \
			cp config.json ../../target/$(TARGET)/release/lib$$(basename $$project).* ../../dist/); \
			(cd ./dist && zip -r $$(basename $$project).plugin lib$$(basename $$project).* config.json);\
	done

# 多目标构建任务
multi-target-build:
	@for target in $(TARGETS); do \
		$(MAKE) build TARGET=$$target; \
		$(MAKE) zip_plugin TARGET=$$target; \
	done

reset:
	@rm -rf dist/lib*.* dist/config.json

clean:
	@cargo clean
	@rm -rf dist

run:
	@cargo run --release
