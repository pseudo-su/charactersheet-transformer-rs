// mod app_error;
extern crate pdfium_render;

use pdfium_render::{prelude::*};
// use app_error::AppError;

use std::collections::HashMap;
use std::ffi::c_int;
use std::fmt::Display;
use std::ptr::null;
use chrono::prelude::*;

// fn pdfium_cfg_static() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
//     Pdfium::bind_to_statically_linked_library()
// }

fn pdfium_cfg_dynamic() -> Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./libs")))
            .or_else(|_| Pdfium::bind_to_system_library())
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdfium = Pdfium::new(pdfium_cfg_dynamic().unwrap());

    // let filled_doc = pdfium.load_pdf_from_file("output/charactersheet.grayscale.pdf", None)?;
    let filled_doc = pdfium.load_pdf_from_file("fixtures/charactersheet.color.pdf", None)?;
    let output_color_template_doc = pdfium.load_pdf_from_file("fixtures/charactersheet.color.template.pdf", None)?;
    let output_grayscale_template_doc = pdfium.load_pdf_from_file("fixtures/charactersheet.grayscale.template.pdf", None)?;

    pdf_describe_file(&filled_doc)?;
    pdf_describe_form(&filled_doc)?;

    pdf_describe_form(&output_color_template_doc)?;
    pdf_describe_form(&output_grayscale_template_doc)?;

    let output_color_initial_bytes = output_color_template_doc.save_to_bytes()?;
    let output_color_doc = pdf_duplicate(&pdfium, &output_color_initial_bytes)?;
    
    let output_grayscale_initial_bytes = output_grayscale_template_doc.save_to_bytes()?;
    let output_grayscale_doc = pdf_duplicate(&pdfium, &output_grayscale_initial_bytes)?;

    pdf_describe_annotations(&filled_doc)?;

    // pdf_set_contents_test(&output_doc)?;
    // output_doc.save_to_file("output/charactersheet.grayscale.pdf")?;

    // pdf_fill_test(&output_doc)?;
    // output_doc.save_to_file("output/modified_document_1.pdf")?;

    let filled_form_data = filled_doc.form()
        .map(|f| f.field_values(filled_doc.pages()))
        .ok_or("")?;

    pdf_fill_form_data(&output_color_doc, &filled_form_data)?;
    output_color_doc.save_to_file("output/charactersheet.grayscale.pdf")?;

    pdf_fill_form_data(&output_grayscale_doc, &filled_form_data)?;
    output_grayscale_doc.save_to_file("output/charactersheet.grayscale.pdf")?;

    Ok(())
}

fn pdf_duplicate<'a: 'c, 'b: 'c, 'c>(pdfium: &'a Pdfium, bytes: &'b [u8]) -> Result<Box<PdfDocument<'c>>, Box<dyn std::error::Error>> {
    let new_document = pdfium.load_pdf_from_byte_slice(
        bytes,
        None,
    )?;

    Ok(Box::new(new_document))
}

fn pdf_describe_file<'a>(document: &'a PdfDocument) -> Result<(), Box<dyn std::error::Error>> {

    println!("PDF file version: {:#?}", document.version());

    println!("PDF page mode: {:#?}", document.pages().page_mode());

    println!("PDF metadata tags:");

    document
        .metadata()
        .iter()
        .enumerate()
        .for_each(|(index, tag)| println!("{}: {:#?} = {}", index, tag.tag_type(), tag.value()));

    Ok(())
}

fn pdf_describe_form<'a>(document: &'a PdfDocument) -> Result<(), Box<dyn std::error::Error>> {

    match document.form() {
        Some(form) => {
            println!(
                "PDF contains an embedded form of type {:#?}",
                form.form_type()
            );
            println!(
                "Field count: {:#?}",
                form.field_values(document.pages()).len()
            )
        },
        None => println!("PDF does not contain an embedded form"),
    };

    Ok(())
}

fn pdf_describe_annotations<'a>(document: &'a PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
    for (page_index, page) in document.pages().iter().enumerate() {
        // For each page in the document, iterate over the annotations attached to that page.

        println!("=============== Page {} ===============", page_index);

        for (annotation_index, annotation) in page.annotations().iter().enumerate() {
            if annotation.contents().is_none() {
                continue;
            }
            println!(
                "Annotation {} is of type {:?} with bounds {:?}",
                annotation_index,
                annotation.annotation_type(),
                annotation.bounds()
            );

            println!(
                "Annotation {} text: {:?}",
                annotation_index,
                page.text().unwrap().for_annotation(&annotation).ok()
            );

            println!(
                "Annotation {} name: {:?}",
                annotation_index,
                annotation.name()
            );

            println!(
                "Annotation {} contents: {:?}",
                annotation_index,
                annotation.contents()
            );

            println!(
                "Annotation {} author: {:?}",
                annotation_index,
                annotation.creator()
            );

            println!(
                "Annotation {} created: {:?}",
                annotation_index,
                annotation.creation_date()
            );

            println!(
                "Annotation {} last modified: {:?}",
                annotation_index,
                annotation.modification_date()
            );

            println!(
                "Annotation {} contains {} page objects",
                annotation_index,
                annotation.objects().len()
            );

            for (object_index, object) in annotation.objects().iter().enumerate() {
                println!(
                    "Annotation {} page object {} is of type {:?}",
                    annotation_index,
                    object_index,
                    object.object_type()
                );

                println!(
                    "Bounds: {:?}, width: {:?}, height: {:?}",
                    object.bounds()?,
                    object.width()?,
                    object.height()?
                );

                // For text objects, we take the extra step of outputting the text
                // contained by the object.

                if let Some(object) = object.as_text_object() {
                    println!(
                        "Text: {} {}-pt {:?}: \"{}\"",
                        object.font().name(),
                        object.unscaled_font_size().value,
                        object.font().weight()?,
                        object.text()
                    );
                }
            }
        }
    }
    Ok(())
}

fn pdf_set_contents_test<'a>(document: &'a PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
    for mut page in document.pages().iter() {
            for mut annotation in page.annotations_mut().iter() {
                annotation.set_contents("test")?;
                annotation.as_widget_annotation_mut()
                    .and_then(|w| {
                        w.form_field_mut()
                    })
                    .and_then(|ff| {
                        ff.as_text_field()
                    })
                    .and_then(|tff| {
                        // tff.bindings().FPDF_SetFormFieldHighlightAlpha(handle, alpha)
                        Some(())
                    });
            }
    }
    Ok(())
}

fn pdf_fill_test<'a>(document: &'a PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
    // let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let pages = document.pages();

    for page in pages.iter() {
        let page_handle = document.bindings().get_handle_from_page(&page);
        let annotations = page.annotations();
        let mut form_fill_info = annotations.bindings().create_formfillinfo(1);
        let form_fill_info_ptr = &mut form_fill_info as *mut _;
        let b = annotations.bindings();

        // See: https://pdfium.googlesource.com/pdfium/+/refs/heads/main/samples/simple_no_v8.c
        let form_handle = annotations.bindings().FPDFDOC_InitFormFillEnvironment(
            document.bindings().get_handle_from_document(&document),
            form_fill_info_ptr,
        );
        b.FORM_OnAfterLoadPage(page_handle, form_handle);

        let c_int_index = (0..).map(|i| i as c_int);
        for (annotation_index, annotation) in c_int_index.zip(annotations.iter()) {
            if annotation.annotation_type() == PdfPageAnnotationType::Widget {
                let annotation_handle = b.FPDFPage_GetAnnot(page_handle, annotation_index);

                b.FPDFAnnot_SetStringValue_str(
                    annotation_handle,
                    "M",
                    "",
                    // &date_time_to_pdf_string(Utc::now()),
                );
                b.FPDFAnnot_SetStringValue_str(annotation_handle, "V", "50");

                b.FPDFAnnot_SetAP(annotation_handle, PdfAppearanceMode::Normal as i32, null());
                b.FPDFPage_CloseAnnot(annotation_handle);
            }
        }

        b.FORM_OnBeforeClosePage(page_handle, form_handle);

        b.FPDFDOC_ExitFormFillEnvironment(form_handle);
        b.FPDFPage_GenerateContent(page_handle);
    }

    Ok(())
}

fn pdf_fill_form_data<'a>(document: &'a PdfDocument, formdata: &HashMap<String, Option<String>>) -> Result<(), Box<dyn std::error::Error>> {

    // let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let pages = document.pages();

    for page in pages.iter() {
        let page_handle = document.bindings().get_handle_from_page(&page);
        let annotations = page.annotations();
        let mut form_fill_info = annotations.bindings().create_formfillinfo(1);
        let form_fill_info_ptr = &mut form_fill_info as *mut _;
        let b = annotations.bindings();

        // See: https://pdfium.googlesource.com/pdfium/+/refs/heads/main/samples/simple_no_v8.c
        let form_handle = annotations.bindings().FPDFDOC_InitFormFillEnvironment(
            document.bindings().get_handle_from_document(&document),
            form_fill_info_ptr,
        );
        b.FORM_OnAfterLoadPage(page_handle, form_handle);

        let c_int_index = (0..).map(|i| i as c_int);
        for (annotation_index, annotation) in c_int_index.zip(annotations.iter()) {
            if annotation.annotation_type() == PdfPageAnnotationType::Widget {
                let annotation_handle = b.FPDFPage_GetAnnot(page_handle, annotation_index);
                let name = annotation.as_form_field()
                    .map(|ff| ff.as_text_field()).unwrap_or(None)
                    .map(|tf| tf.name()).unwrap_or(None);
                if name == None { continue; }
                let value = formdata.get(&name.clone().unwrap()).unwrap_or(&None).to_owned();
                if value == None { continue; }

                println!("setting: {}={}", name.unwrap(), value.clone().unwrap());

                b.FPDFAnnot_SetStringValue_str(
                    annotation_handle,
                    "M",
                    &date_time_to_pdf_string(Utc::now()),
                );
                b.FPDFAnnot_SetStringValue_str(annotation_handle, "V", &value.unwrap());

                b.FPDFAnnot_SetAP(annotation_handle, PdfAppearanceMode::Normal as i32, null());
                b.FPDFPage_CloseAnnot(annotation_handle);
            }
        }

        b.FORM_OnBeforeClosePage(page_handle, form_handle);

        b.FPDFDOC_ExitFormFillEnvironment(form_handle);
        b.FPDFPage_GenerateContent(page_handle);
    }

    Ok(())
}

fn date_time_to_pdf_string<T, O>(date: DateTime<T>) -> String
where
    T: TimeZone<Offset = O>,
    O: Display,
{
    let date_part = date.format("%Y%m%d%H%M%S");

    let timezone_part = format!("{}'", date.format("%:z"))
        .replace("+00:00'", "Z00'00'")
        .replace(':', "'");

    format!("D:{}{}", date_part, timezone_part)
}
