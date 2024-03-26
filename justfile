
default:
	just -l

test:
	smdk build
	smdk test --text "2024/12/02 01:13:23"


