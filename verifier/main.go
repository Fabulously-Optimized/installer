package main

import (
	"C"
	"encoding/hex"

	"github.com/sigstore/sigstore-go/pkg/bundle"
	"github.com/sigstore/sigstore-go/pkg/fulcio/certificate"
	"github.com/sigstore/sigstore-go/pkg/root"
	"github.com/sigstore/sigstore-go/pkg/tuf"
	"github.com/sigstore/sigstore-go/pkg/verify"
)

func main() {}

//export verifier_verify
func verifier_verify(digestHexRaw *C.char, bundlePathRaw *C.char) *C.char {
	digestHex := C.GoString(digestHexRaw)
	bundlePath := C.GoString(bundlePathRaw)

	opts := tuf.DefaultOptions()

	trustedMaterial, err := root.NewLiveTrustedRoot(opts)
	if err != nil {
		return C.CString(err.Error())
	}

	sev, err := verify.NewSignedEntityVerifier(trustedMaterial, verify.WithSignedCertificateTimestamps(1), verify.WithTransparencyLog(1), verify.WithObserverTimestamps(1))
	if err != nil {
		return C.CString(err.Error())
	}

	digest, err := hex.DecodeString(digestHex)
	if err != nil {
		return C.CString(err.Error())
	}

	sanMatcher, err := verify.NewSANMatcher("", "^https://github.com/Fabulously-Optimized/fabulously-optimized/")
	if err != nil {
		return C.CString(err.Error())
	}
	issuerMatcher, err := verify.NewIssuerMatcher("https://token.actions.githubusercontent.com asdf", "")
	if err != nil {
		return C.CString(err.Error())
	}
	certID, err := verify.NewCertificateIdentity(sanMatcher, issuerMatcher, certificate.Extensions{
		BuildTrigger:                        "release",
		SourceRepositoryURI:                 "https://github.com/Fabulously-Optimized/fabulously-optimized",
		RunnerEnvironment:                   "github-hosted",
		SourceRepositoryVisibilityAtSigning: "public",
	})
	if err != nil {
		return C.CString(err.Error())
	}

	b, err := bundle.LoadJSONFromPath(bundlePath)
	if err != nil {
		return C.CString(err.Error())
	}

	_, err = sev.Verify(b, verify.NewPolicy(verify.WithArtifactDigest("sha256", digest), verify.WithCertificateIdentity(certID)))
	if err != nil {
		return C.CString(err.Error())
	}

	return nil
}
