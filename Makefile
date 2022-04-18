all: report.pdf

report.pdf: report.md sources.bib
	pandoc report.md --template eisvogel --filter pandoc-fignos --listings --citeproc -o report.pdf
