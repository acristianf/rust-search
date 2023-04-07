console.log("Querying api/search");

const inputElement = document.getElementById('query');

inputElement.addEventListener('input', function() {
	fetch("api/search", {
		method: "POST",
		headers: {
			"Content-Type": "text/plain",
		},
		body: inputElement.value,
	})
})

