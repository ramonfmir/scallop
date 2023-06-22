UNAME_S := $(shell uname -s)
UNAME_P := $(shell uname -p)

ifeq ($(UNAME_S),Darwin)
	ifneq ($(filter arm%,$(UNAME_P)),) 
		COMMAND = maturin build --release --target aarch64-apple-darwin
	else
		COMMAND = maturin develop --release
	endif
else
	COMMAND = maturin develop --release
endif

all:
	$(COMMAND)

test:
	python3 tests/test.py

clean:
	rm -f scallopy/scallopy.cpython*
	rm -rf tests/*.sclcmpl
