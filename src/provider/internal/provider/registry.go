package provider

var registry = map[string]func() Provider{
	"dropbox": func() Provider { return &DropboxProvider{} },
	"s3":      func() Provider { return &S3Provider{} },
}

func Get(name string) (Provider, bool) {
	fn, ok := registry[name]
	if !ok {
		return nil, false
	}
	return fn(), true
}

func List() []string {
	var names []string
	for name := range registry {
		names = append(names, name)
	}
	return names
}
