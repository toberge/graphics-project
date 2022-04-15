all: report.pdf

report.pdf: report.md sources.bib
	pandoc report.md --template eisvogel --listings --citeproc -o report.pdf
