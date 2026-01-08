// Creator page JavaScript

let providerModels = {};
let currentProvider = '';
let currentModel = '';
let inputs = [];
let isRestoring = false;

const STORAGE_KEY = 'promptcmd_creator_state';

function initProviderModels(data) {
    providerModels = data;
}

function saveState() {
    if (isRestoring) return;

    try {
        const promptName = document.getElementById('promptName').value;
        const promptContent = generatePromptContent();

        const state = {
            name: promptName,
            content: promptContent
        };

        localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch (e) {
        console.error('Failed to save state:', e);
    }
}

function loadState() {
    try {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (!saved) return false;

        const state = JSON.parse(saved);
        if (!state || !state.content) return false;

        isRestoring = true;

        // Parse the prompt content
        const lines = state.content.split('\n');
        let inFrontmatter = false;
        let frontmatterLines = [];
        let templateLines = [];
        let frontmatterEnded = false;

        for (const line of lines) {
            if (line.trim() === '---') {
                if (!inFrontmatter) {
                    inFrontmatter = true;
                } else {
                    inFrontmatter = false;
                    frontmatterEnded = true;
                }
                continue;
            }

            if (inFrontmatter) {
                frontmatterLines.push(line);
            } else if (frontmatterEnded) {
                templateLines.push(line);
            }
        }

        // Parse frontmatter
        let parsedModel = '';
        let parsedOutputFormat = 'text';
        let parsedInputs = [];
        let inInputSchema = false;

        for (const line of frontmatterLines) {
            const trimmed = line.trim();

            if (trimmed.startsWith('model:')) {
                parsedModel = trimmed.substring(6).trim();
            } else if (trimmed.startsWith('format:')) {
                parsedOutputFormat = trimmed.substring(7).trim();
            } else if (trimmed === 'schema:') {
                inInputSchema = true;
            } else if (inInputSchema && trimmed && !trimmed.startsWith('input:') && !trimmed.startsWith('output:')) {
                // Parse input line: name?: type, description
                const match = trimmed.match(/^(\w+)(\?)?:\s*(\w+)(?:,\s*(.+))?$/);
                if (match) {
                    parsedInputs.push({
                        name: match[1],
                        required: !match[2],
                        type: match[3],
                        description: match[4] || ''
                    });
                }
            } else if (trimmed.startsWith('output:')) {
                inInputSchema = false;
            }
        }

        // Restore prompt name
        document.getElementById('promptName').value = state.name || 'prompt';

        // Restore model
        if (parsedModel) {
            let restoredProvider = '';
            let restoredModel = '';

            if (parsedModel.includes('/')) {
                // Has slash - validate both parts
                const parts = parsedModel.split('/');
                const providerPart = parts[0];
                const modelPart = parts[1];

                // Check if provider exists and model exists in that provider
                if (providerModels[providerPart] && providerModels[providerPart].models.includes(modelPart)) {
                    // Both are valid
                    restoredProvider = providerPart;
                    restoredModel = modelPart;
                } else {
                    // Invalid provider or model - treat as manual entry
                    restoredProvider = 'other';
                    restoredModel = parsedModel;
                }
            } else {
                // No slash - check if it's a known provider
                if (providerModels[parsedModel]) {
                    // It's a known provider without a specific model
                    restoredProvider = parsedModel;
                    restoredModel = '';
                } else {
                    // It's a manual "other" entry
                    restoredProvider = 'other';
                    restoredModel = parsedModel;
                }
            }

            // Set provider dropdown
            const providerSelect = document.getElementById('providerSelect');
            providerSelect.value = restoredProvider;
            currentProvider = restoredProvider;
            updateProvider();

            // Set model dropdown or input and update currentModel
            if (restoredProvider === 'other') {
                document.getElementById('modelInput').value = restoredModel;
                currentModel = restoredModel;
            } else if (restoredModel) {
                const modelSelect = document.getElementById('modelSelect');
                modelSelect.value = restoredModel;
                currentModel = restoredModel;
            }
        }

        // Restore output format
        document.getElementById('outputFormat').value = parsedOutputFormat;

        // Restore inputs
        inputs = parsedInputs;
        renderInputs();

        // Restore template
        document.getElementById('templateEditor').value = templateLines.join('\n');

        updatePreview();

        isRestoring = false;
        return true;
    } catch (e) {
        console.error('Failed to load state:', e);
        isRestoring = false;
        return false;
    }
}

function updateProvider() {
    const provider = document.getElementById('providerSelect').value;
    currentProvider = provider;

    const modelSelect = document.getElementById('modelSelect');
    const modelInput = document.getElementById('modelInput');

    if (provider === '') {
        // None/Default selected - show disabled model selector
        modelSelect.style.display = 'block';
        modelInput.style.display = 'none';
        modelSelect.innerHTML = '<option value="">None/Default</option>';
        modelSelect.disabled = true;
        currentModel = '';
    } else if (provider === 'other') {
        modelSelect.style.display = 'none';
        modelInput.style.display = 'block';
        modelSelect.disabled = false;
        modelInput.value = '';
        currentModel = '';
        modelInput.focus();
    } else {
        modelSelect.style.display = 'block';
        modelInput.style.display = 'none';
        modelSelect.disabled = false;

        // Populate model dropdown
        const models = providerModels[provider].models;
        modelSelect.innerHTML = '<option value="">None/Default</option>' +
            models.map(m => `<option value="${m}">${m}</option>`).join('');

        currentModel = '';
    }

    updatePreview();
    saveState();
}

function updateModel() {
    const modelSelect = document.getElementById('modelSelect');
    const modelInput = document.getElementById('modelInput');

    if (currentProvider === 'other') {
        currentModel = modelInput.value;
    } else {
        currentModel = modelSelect.value;
    }

    updatePreview();
    saveState();
}

function getModelString() {
    if (!currentProvider) {
        return '';
    }
    if (currentProvider == "other") {
      return currentModel;
    }
    if (currentModel == "") {
      return currentProvider;
    } else {
        return `${currentProvider}/${currentModel}`;
    }
}

function addInput() {
    const input = {
        name: `input${inputs.length + 1}`,
        type: 'string',
        description: '',
        required: true
    };
    inputs.push(input);
    renderInputs();
    updatePreview();
    saveState();
}

function removeInput(index) {
    inputs.splice(index, 1);
    renderInputs();
    updatePreview();
    saveState();
}

function updateInputField(index, field, value) {
    inputs[index][field] = value;
    updatePreview();
    saveState();
}

function renderInputs() {
    const inputList = document.getElementById('inputList');

    if (inputs.length === 0) {
        inputList.innerHTML = '<div class="empty-state">No inputs yet. Click "Add Input" to get started.</div>';
        return;
    }

    inputList.innerHTML = inputs.map((input, index) => `
        <div class="input-item">
            <div class="input-item-field">
                <span class="input-item-label">Name & Description</span>
                <input type="text"
                       value="${escapeHtml(input.name)}"
                       placeholder="Input name"
                       oninput="updateInputField(${index}, 'name', this.value)">
                <input type="text"
                       value="${escapeHtml(input.description)}"
                       placeholder="Description (optional)"
                       oninput="updateInputField(${index}, 'description', this.value)">
            </div>
            <div class="input-item-field">
                <span class="input-item-label">Type</span>
                <select onchange="updateInputField(${index}, 'type', this.value)">
                    <option value="string" ${input.type === 'string' ? 'selected' : ''}>string</option>
                    <option value="number" ${input.type === 'number' ? 'selected' : ''}>number</option>
                    <option value="boolean" ${input.type === 'boolean' ? 'selected' : ''}>boolean</option>
                    <option value="array" ${input.type === 'array' ? 'selected' : ''}>array</option>
                    <option value="object" ${input.type === 'object' ? 'selected' : ''}>object</option>
                </select>
            </div>
            <div class="checkbox-wrapper">
                <span class="input-item-label">Required</span>
                <input type="checkbox"
                       ${input.required ? 'checked' : ''}
                       onchange="updateInputField(${index}, 'required', this.checked)">
            </div>
            <button class="delete-btn" onclick="removeInput(${index})" title="Remove input">×</button>
        </div>
    `).join('');
}

function generatePromptContent() {
    const modelString = getModelString();
    const outputFormat = document.getElementById('outputFormat').value;
    const template = document.getElementById('templateEditor').value;

    let frontmatter = '---\n';

    // Add model if specified
    if (modelString) {
        frontmatter += `model: ${modelString}\n`;
    }

    // Add input schema if there are inputs
    if (inputs.length > 0) {
        frontmatter += 'input:\n  schema:\n';
        inputs.forEach(input => {
            const name = input.name || 'unnamed';
            const suffix = input.required ? '' : '?';
            const desc = input.description ? `, ${input.description}` : '';
            frontmatter += `    ${name}${suffix}: ${input.type}${desc}\n`;
        });
    }

    // Add output format
    frontmatter += 'output:\n';
    frontmatter += `  format: ${outputFormat}\n`;
    frontmatter += '---\n';

    return frontmatter + (template || '');
}

function highlightYAML(content) {
    const lines = content.split('\n');
    let inFrontmatter = false;
    let frontmatterEnded = false;

    return lines.map(line => {
        // Check for frontmatter delimiters
        if (line.trim() === '---') {
            if (!inFrontmatter) {
                inFrontmatter = true;
                return `<span class="yaml-comment">${escapeHtml(line)}</span>`;
            } else {
                inFrontmatter = false;
                frontmatterEnded = true;
                return `<span class="yaml-comment">${escapeHtml(line)}</span>`;
            }
        }

        // Template content (after frontmatter)
        if (frontmatterEnded) {
            return `<span class="template-text">${escapeHtml(line)}</span>`;
        }

        // YAML frontmatter
        if (inFrontmatter) {
            // Check for comments
            if (line.trim().startsWith('#')) {
                return `<span class="yaml-comment">${escapeHtml(line)}</span>`;
            }

            // Key-value pairs
            const keyValueMatch = line.match(/^(\s*)([^:]+):\s*(.*)$/);
            if (keyValueMatch) {
                const indent = keyValueMatch[1];
                const key = keyValueMatch[2];
                const value = keyValueMatch[3];

                if (value) {
                    return `${indent}<span class="yaml-key">${escapeHtml(key)}:</span> <span class="yaml-value">${escapeHtml(value)}</span>`;
                } else {
                    return `${indent}<span class="yaml-key">${escapeHtml(key)}:</span>`;
                }
            }
        }

        return escapeHtml(line);
    }).join('\n');
}

function updatePreview() {
    const content = generatePromptContent();
    const highlighted = highlightYAML(content);
    document.getElementById('preview').innerHTML = highlighted;
}

function downloadPrompt() {
    const content = generatePromptContent();
    const promptName = document.getElementById('promptName').value.trim() || 'prompt';
    const filename = promptName.endsWith('.prompt') ? promptName : `${promptName}.prompt`;

    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function copyPrompt() {
    const content = generatePromptContent();
    const btn = document.getElementById('copyBtn');
    copyToClipboard(content, btn);
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    if (typeof window.providerData !== 'undefined') {
        initProviderModels(window.providerData);
    }

    // Set up model selector as disabled with "None/Default" since empty string is selected by default
    const modelSelect = document.getElementById('modelSelect');
    modelSelect.innerHTML = '<option value="">None/Default</option>';
    modelSelect.disabled = true;
    document.getElementById('modelInput').style.display = 'none';

    // Try to load saved state
    const stateLoaded = loadState();

    if (!stateLoaded) {
        updatePreview();
    }

    // Add event listeners to save state on change
    document.getElementById('promptName').addEventListener('input', saveState);
    document.getElementById('outputFormat').addEventListener('change', saveState);
    document.getElementById('templateEditor').addEventListener('input', saveState);
});
