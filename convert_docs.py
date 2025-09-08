#!/usr/bin/env python3
"""
Convert Markdown to DOCX and PDF
Requires: pip install python-docx markdown2 pdfkit
"""

import markdown2
from docx import Document
from docx.shared import Inches
import os

def markdown_to_docx(md_file_path, docx_file_path):
    """Convert Markdown file to DOCX"""
    
    # Read markdown file
    with open(md_file_path, 'r', encoding='utf-8') as file:
        md_content = file.read()
    
    # Convert markdown to HTML
    html = markdown2.markdown(md_content, extras=['fenced-code-blocks', 'tables'])
    
    # Create Word document
    doc = Document()
    
    # Add title
    title = doc.add_heading('MetaPilot - Complete Guide for Absolute Beginners', 0)
    
    # Simple conversion (basic - for better results use pandoc)
    lines = md_content.split('\n')
    
    for line in lines:
        line = line.strip()
        if not line:
            continue
        
        # Handle headers
        if line.startswith('#'):
            level = len(line) - len(line.lstrip('#'))
            text = line.lstrip('#').strip()
            if level <= 3:
                doc.add_heading(text, level)
            else:
                p = doc.add_paragraph()
                p.add_run(text).bold = True
        
        # Handle code blocks
        elif line.startswith('```'):
            continue
        
        # Handle lists
        elif line.startswith('- ') or line.startswith('* '):
            text = line[2:].strip()
            doc.add_paragraph(text, style='List Bullet')
        
        # Handle numbered lists
        elif line[0:3].replace('.', '').replace(' ', '').isdigit():
            text = line.split('.', 1)[1].strip() if '.' in line else line
            doc.add_paragraph(text, style='List Number')
        
        # Regular paragraphs
        else:
            if not line.startswith('#'):
                doc.add_paragraph(line)
    
    # Save document
    doc.save(docx_file_path)
    print(f"✅ DOCX saved to: {docx_file_path}")

def main():
    # File paths
    md_file = "METAPILOT_COMPREHENSIVE_DOCUMENTATION.md"
    docx_file = "METAPILOT_COMPREHENSIVE_DOCUMENTATION.docx"
    
    # Check if markdown file exists
    if not os.path.exists(md_file):
        print(f"❌ Markdown file not found: {md_file}")
        return
    
    try:
        # Convert to DOCX
        markdown_to_docx(md_file, docx_file)
        print("🎉 Conversion completed successfully!")
        
        print("\n📋 Next steps:")
        print("1. Open the DOCX file to review formatting")
        print("2. Make any manual adjustments needed")
        print("3. Save as PDF from Word if needed")
        
    except Exception as e:
        print(f"❌ Error during conversion: {e}")
        print("\n💡 Alternative options:")
        print("1. Use pandoc: pandoc input.md -o output.docx")
        print("2. Use online converter: pandoc.org/try")
        print("3. Copy-paste into Word/Google Docs")

if __name__ == "__main__":
    main()