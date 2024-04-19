
default:
	just -l

code-test:
	cargo test

smdk-test:
	smdk build
	# smdk test --text '{ "foo":"2024/12/02 01:13:23" }'
	smdk test \
		--text "2024/12/02 01:13:23" \
		--params date_ops_params='{"source_format":"%Y/%d/%m %H:%M:%S","output_format":"%Y-%d-%m %H:%M:%S","source_timezone":2,"output_timezone":"UTC","fields":["foo"]}'

# different source time zone
smdk-test2:
	smdk build
	# smdk test --text '{ "foo":"2024/12/02 01:13:23" }'
	smdk test \
		--text "2024/12/02 01:13:23" \
		--params date_ops_params='{"source_format":"%Y/%d/%m %H:%M:%S","output_format":"%Y-%d-%m %H:%M:%S","source_timezone":5,"output_timezone":"UTC","fields":["foo"]}'



