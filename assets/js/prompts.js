// Prompts page JavaScript
// Note: currentProvider and currentModel are defined in prompts.html inline script

function updateProvider() {
    const provider = document.getElementById('providerSelect').value;
    currentProvider = provider;

    const modelSelect = document.getElementById('modelSelect');
    const modelInput = document.getElementById('modelInput');

    if (provider === 'none') {
        // Show disabled "None/Default" in model selector
        modelSelect.style.display = 'block';
        modelInput.style.display = 'none';
        modelSelect.innerHTML = '<option value="">None/Default</option>';
        modelSelect.disabled = true;
        currentModel = null;
    } else if (provider === 'other') {
        modelSelect.style.display = 'none';
        modelInput.style.display = 'block';
        modelSelect.disabled = false;
        modelInput.focus();
        currentModel = modelInput.value;
    } else {
        modelSelect.style.display = 'block';
        modelInput.style.display = 'none';
        modelSelect.disabled = false;

        // Populate model dropdown
        const models = window.providerData[provider].models;
        modelSelect.innerHTML = '<option value="none">None/Default</option>'
          + models.map(m =>
            `<option value="${m}">${m}</option>`
        ).join('');

        currentModel = getModelString(provider, "none");
    }

    renderAllPrompts();
}

function updateModel() {
    const modelSelect = document.getElementById('modelSelect');
    const modelInput = document.getElementById('modelInput');


    if (currentProvider === 'other') {
        currentModel = modelInput.value;
    } else {
        const selectedModel = modelSelect.value;
        currentModel = getModelString(currentProvider, selectedModel);
    }

    renderAllPrompts();
}

function getModelString(provider, model) {
    if (model == "none") {
      return provider;
    } else {
        return `${provider}/${model}`;
    }
}
