(function() {var implementors = {};
implementors["propagate"] = [{"text":"impl&lt;T, E, S, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/try_trait/trait.FromResidual.html\" title=\"trait core::ops::try_trait::FromResidual\">FromResidual</a>&lt;<a class=\"enum\" href=\"propagate/enum.Result.html\" title=\"enum propagate::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E, S&gt;&gt; for <a class=\"enum\" href=\"propagate/enum.Result.html\" title=\"enum propagate::Result\">Result</a>&lt;T, F, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"propagate/trait.Traced.html\" title=\"trait propagate::Traced\">Traced</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;E&gt;,&nbsp;</span>","synthetic":false,"types":["propagate::result::Result"]},{"text":"impl&lt;T, E, S, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/try_trait/trait.FromResidual.html\" title=\"trait core::ops::try_trait::FromResidual\">FromResidual</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;&gt; for <a class=\"enum\" href=\"propagate/enum.Result.html\" title=\"enum propagate::Result\">Result</a>&lt;T, F, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"propagate/trait.Traced.html\" title=\"trait propagate::Traced\">Traced</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;E&gt;,&nbsp;</span>","synthetic":false,"types":["propagate::result::Result"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()