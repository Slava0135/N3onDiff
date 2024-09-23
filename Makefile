build: create-harness build-neo-go build-neo-sharp

create-harness:
	@mkdir -p harness

build-neo-go:
	@echo '=> Building NeoGo...'
	@make -C neo-go build
	@cp neo-go/bin/neo-go harness/neo-go

build-neo-sharp:
	@echo '=> Building NeoSharp...'
	@cd neo/VmHarness && dotnet publish -c release -o bin/harness
	@cp neo/VmHarness/bin/harness/VmHarness harness/neo-sharp
