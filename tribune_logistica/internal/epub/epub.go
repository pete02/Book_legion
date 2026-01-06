package epub

type Epub struct {
	Path  string
	Spine []SpineItem
	Nav   []PrettySpineItem
}

func New(path string) (Epub, error) {
	spine, err := LoadSpine(path)
	if err != nil {
		return Epub{}, err
	}
	epub := Epub{
		Path:  path,
		Spine: spine,
	}
	nav, err := epub.LoadPrettySpine()
	if err != nil {
		return epub, err
	}

	epub.Nav = nav

	return epub, nil
}
