package main

import (
	"encoding/hex"
	"os"

	"github.com/sigstore/sigstore-go/pkg/bundle"
	"github.com/sigstore/sigstore-go/pkg/fulcio/certificate"
	"github.com/sigstore/sigstore-go/pkg/root"
	"github.com/sigstore/sigstore-go/pkg/tuf"
	"github.com/sigstore/sigstore-go/pkg/verify"
)

func main() {
	digestHex := os.Args[1]
	bundlePath := os.Args[2]

	opts := tuf.DefaultOptions()

	trustedMaterial, err := root.NewLiveTrustedRoot(opts)
	if err != nil {
		panic(err)
	}

	sev, err := verify.NewSignedEntityVerifier(trustedMaterial, verify.WithSignedCertificateTimestamps(1), verify.WithTransparencyLog(1), verify.WithObserverTimestamps(1))
	if err != nil {
		panic(err)
	}

	digest, err := hex.DecodeString(digestHex)
	if err != nil {
		panic(err)
	}

	sanMatcher, err := verify.NewSANMatcher("", "^https://github.com/Fabulously-Optimized/fabulously-optimized/")
	if err != nil {
		panic(err)
	}
	issuerMatcher, err := verify.NewIssuerMatcher("https://token.actions.githubusercontent.com", "")
	if err != nil {
		panic(err)
	}
	certID, err := verify.NewCertificateIdentity(sanMatcher, issuerMatcher, certificate.Extensions{
		BuildTrigger:                        "release",
		SourceRepositoryURI:                 "https://github.com/Fabulously-Optimized/fabulously-optimized",
		RunnerEnvironment:                   "github-hosted",
		SourceRepositoryVisibilityAtSigning: "public",
	})
	if err != nil {
		panic(err)
	}

	b, err := bundle.LoadJSONFromPath(bundlePath)
	if err != nil {
		panic(err)
	}

	_, err = sev.Verify(b, verify.NewPolicy(verify.WithArtifactDigest("sha256", digest), verify.WithCertificateIdentity(certID)))
	if err != nil {
		panic(err)
	}
}
