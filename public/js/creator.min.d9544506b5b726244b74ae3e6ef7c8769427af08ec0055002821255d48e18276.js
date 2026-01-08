let providerModels={},currentProvider="",currentModel="",inputs=[],isRestoring=!1;const STORAGE_KEY="promptcmd_creator_state";function initProviderModels(e){providerModels=e}function saveState(){if(isRestoring)return;try{const e=document.getElementById("promptName").value,t=generatePromptContent(),n={name:e,content:t};localStorage.setItem(STORAGE_KEY,JSON.stringify(n))}catch(e){console.error("Failed to save state:",e)}}function loadState(){try{const o=localStorage.getItem(STORAGE_KEY);if(!o)return!1;const t=JSON.parse(o);if(!t||!t.content)return!1;isRestoring=!0;const d=t.content.split(`
`);let n=!1,i=[],a=[],r=!1;for(const e of d){if(e.trim()==="---"){n?(n=!1,r=!0):n=!0;continue}n?i.push(e):r&&a.push(e)}let e="",c="text",l=[],s=!1;for(const n of i){const t=n.trim();if(t.startsWith("model:"))e=t.substring(6).trim();else if(t.startsWith("format:"))c=t.substring(7).trim();else if(t==="schema:")s=!0;else if(s&&t&&!t.startsWith("input:")&&!t.startsWith("output:")){const e=t.match(/^(\w+)(\?)?:\s*(\w+)(?:,\s*(.+))?$/);e&&l.push({name:e[1],required:!e[2],type:e[3],description:e[4]||""})}else t.startsWith("output:")&&(s=!1)}if(document.getElementById("promptName").value=t.name||"prompt",e){let n="",t="";if(e.includes("/")){const o=e.split("/"),s=o[0],i=o[1];providerModels[s]&&providerModels[s].models.includes(i)?(n=s,t=i):(n="other",t=e)}else providerModels[e]?(n=e,t=""):(n="other",t=e);const s=document.getElementById("providerSelect");if(s.value=n,currentProvider=n,updateProvider(),n==="other")document.getElementById("modelInput").value=t,currentModel=t;else if(t){const e=document.getElementById("modelSelect");e.value=t,currentModel=t}}return document.getElementById("outputFormat").value=c,inputs=l,renderInputs(),document.getElementById("templateEditor").value=a.join(`
`),updatePreview(),isRestoring=!1,!0}catch(e){return console.error("Failed to load state:",e),isRestoring=!1,!1}}function updateProvider(){const n=document.getElementById("providerSelect").value;currentProvider=n;const e=document.getElementById("modelSelect"),t=document.getElementById("modelInput");if(n==="")e.style.display="block",t.style.display="none",e.innerHTML='<option value="">None/Default</option>',e.disabled=!0,currentModel="";else if(n==="other")e.style.display="none",t.style.display="block",e.disabled=!1,t.value="",currentModel="",t.focus();else{e.style.display="block",t.style.display="none",e.disabled=!1;const s=providerModels[n].models;e.innerHTML='<option value="">None/Default</option>'+s.map(e=>`<option value="${e}">${e}</option>`).join(""),currentModel=""}updatePreview(),saveState()}function updateModel(){const e=document.getElementById("modelSelect"),t=document.getElementById("modelInput");currentProvider==="other"?currentModel=t.value:currentModel=e.value,updatePreview(),saveState()}function getModelString(){return currentProvider?currentProvider=="other"?currentModel:currentModel==""?currentProvider:`${currentProvider}/${currentModel}`:""}function addInput(){const e={name:`input${inputs.length+1}`,type:"string",description:"",required:!0};inputs.push(e),renderInputs(),updatePreview(),saveState()}function removeInput(e){inputs.splice(e,1),renderInputs(),updatePreview(),saveState()}function updateInputField(e,t,n){inputs[e][t]=n,updatePreview(),saveState()}function renderInputs(){const e=document.getElementById("inputList");if(inputs.length===0){e.innerHTML='<div class="empty-state">No inputs yet. Click "Add Input" to get started.</div>';return}e.innerHTML=inputs.map((e,t)=>`
        <div class="input-item">
            <div class="input-item-field">
                <span class="input-item-label">Name & Description</span>
                <input type="text"
                       value="${escapeHtml(e.name)}"
                       placeholder="Input name"
                       oninput="updateInputField(${t}, 'name', this.value)">
                <input type="text"
                       value="${escapeHtml(e.description)}"
                       placeholder="Description (optional)"
                       oninput="updateInputField(${t}, 'description', this.value)">
            </div>
            <div class="input-item-field">
                <span class="input-item-label">Type</span>
                <select onchange="updateInputField(${t}, 'type', this.value)">
                    <option value="string" ${e.type==="string"?"selected":""}>string</option>
                    <option value="number" ${e.type==="number"?"selected":""}>number</option>
                    <option value="boolean" ${e.type==="boolean"?"selected":""}>boolean</option>
                    <option value="array" ${e.type==="array"?"selected":""}>array</option>
                    <option value="object" ${e.type==="object"?"selected":""}>object</option>
                </select>
            </div>
            <div class="checkbox-wrapper">
                <span class="input-item-label">Required</span>
                <input type="checkbox"
                       ${e.required?"checked":""}
                       onchange="updateInputField(${t}, 'required', this.checked)">
            </div>
            <button class="delete-btn" onclick="removeInput(${t})" title="Remove input">×</button>
        </div>
    `).join("")}function generatePromptContent(){const t=getModelString(),n=document.getElementById("outputFormat").value,s=document.getElementById("templateEditor").value;let e=`---
`;return t&&(e+=`model: ${t}
`),inputs.length>0&&(e+=`input:
  schema:
`,inputs.forEach(t=>{const n=t.name||"unnamed",s=t.required?"":"?",o=t.description?`, ${t.description}`:"";e+=`    ${n}${s}: ${t.type}${o}
`})),e+=`output:
`,e+=`  format: ${n}
`,e+=`---
`,e+(s||"")}function highlightYAML(e){const s=e.split(`
`);let t=!1,n=!1;return s.map(e=>{if(e.trim()==="---")return t?(t=!1,n=!0,`<span class="yaml-comment">${escapeHtml(e)}</span>`):(t=!0,`<span class="yaml-comment">${escapeHtml(e)}</span>`);if(n)return`<span class="template-text">${escapeHtml(e)}</span>`;if(t){if(e.trim().startsWith("#"))return`<span class="yaml-comment">${escapeHtml(e)}</span>`;const t=e.match(/^(\s*)([^:]+):\s*(.*)$/);if(t){const e=t[1],n=t[2],s=t[3];return s?`${e}<span class="yaml-key">${escapeHtml(n)}:</span> <span class="yaml-value">${escapeHtml(s)}</span>`:`${e}<span class="yaml-key">${escapeHtml(n)}:</span>`}}return escapeHtml(e)}).join(`
`)}function updatePreview(){const e=generatePromptContent(),t=highlightYAML(e);document.getElementById("preview").innerHTML=t}function downloadPrompt(){const s=generatePromptContent(),t=document.getElementById("promptName").value.trim()||"prompt",o=t.endsWith(".prompt")?t:`${t}.prompt`,i=new Blob([s],{type:"text/plain"}),n=URL.createObjectURL(i),e=document.createElement("a");e.href=n,e.download=o,document.body.appendChild(e),e.click(),document.body.removeChild(e),URL.revokeObjectURL(n)}function copyPrompt(){const e=generatePromptContent(),t=document.getElementById("copyBtn");copyToClipboard(e,t)}document.addEventListener("DOMContentLoaded",()=>{typeof window.providerData!="undefined"&&initProviderModels(window.providerData);const e=document.getElementById("modelSelect");e.innerHTML='<option value="">None/Default</option>',e.disabled=!0,document.getElementById("modelInput").style.display="none";const t=loadState();t||updatePreview(),document.getElementById("promptName").addEventListener("input",saveState),document.getElementById("outputFormat").addEventListener("change",saveState),document.getElementById("templateEditor").addEventListener("input",saveState)})