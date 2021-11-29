use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use structopt::StructOpt;
use std::io::{Error, ErrorKind};

const NT_GNU_BUILD_ID: u32 = 3;
const NT_GO_BUILD_ID: u32 = 4;
const MAG_ELF: (u8, u8, u8, u8) = (127, 69, 76, 70);
const PT_NOTE: u32 = 4;

#[derive(Debug, StructOpt)]
#[structopt(name = "elfget", about = "Get a BuildId from a ELF64 file.")]
struct Opt {
    #[structopt(name = "FILE", parse(from_os_str))]
    file_name: PathBuf,
}


fn get_buildid(elf_filename:PathBuf) -> Result<String, io::Error> {
    let mut res = String::from("");
    let mut f = File::open(elf_filename)?;
    let ei_mag0 = f.read_u8()?;
    let ei_mag1 = f.read_u8()?;
    let ei_mag2 = f.read_u8()?;
    let ei_mag3 = f.read_u8()?;
    let ei_class = f.read_u8()?;
    let _ei_data = f.read_u8()?;
    let _ei_version = f.read_u8()?;
    let _ei_osabi = f.read_u8()?;
    let _ei_abiversion = f.read_u8()?;
    let _ei_pad = f.read_u8()?;
    f.read_u8()?;
    f.read_u8()?;
    f.read_u8()?;
    f.read_u8()?;
    f.read_u8()?;
    f.read_u8()?;

    if (ei_mag0, ei_mag1, ei_mag2, ei_mag3) != MAG_ELF {
        return Err(Error::new(ErrorKind::Other, "Not an ELF64 file: wrong header."));
    }

    if ei_class != 2 {
        return Err(Error::new(ErrorKind::Other, "Not an ELF64 file: wrong class."));
    }

    let _e_type = f.read_u16::<LittleEndian>()?;
    let _e_machine = f.read_u16::<LittleEndian>()?;
    let _e_version = f.read_u32::<LittleEndian>()?;
    let _e_entry = f.read_u64::<LittleEndian>()?;
    let e_phoff = f.read_u64::<LittleEndian>()?;
    let _e_shoff = f.read_u64::<LittleEndian>()?;
    let _e_flags = f.read_u32::<LittleEndian>()?;
    let _e_ehsize = f.read_u16::<LittleEndian>()?;
    let _e_phentsize = f.read_u16::<LittleEndian>()?;
    let e_phnum = f.read_u16::<LittleEndian>()?;
    let _e_shentsize = f.read_u16::<LittleEndian>()?;
    let _e_shnum = f.read_u16::<LittleEndian>()?;
    let _e_shstrndx = f.read_u16::<LittleEndian>()?;

    if e_phoff == 0 {
        return Err(Error::new(ErrorKind::Other, "Program headers not found."));
    }

    f.seek(SeekFrom::Start(e_phoff))?;
    for _idx in 0..e_phnum {
        let p_type = f.read_u32::<LittleEndian>()?;
        let _p_flags = f.read_u32::<LittleEndian>()?;
        let p_offset = f.read_u64::<LittleEndian>()?;
        let _p_vaddr = f.read_u64::<LittleEndian>()?;
        let _p_paddr = f.read_u64::<LittleEndian>()?;
        let p_filesz = f.read_u64::<LittleEndian>()?;
        let _p_memsz = f.read_u64::<LittleEndian>()?;
        let _p_align = f.read_u64::<LittleEndian>()?;

        if p_type == PT_NOTE {
            let p_end = p_offset + p_filesz;
            f.seek(SeekFrom::Start(p_offset))?;

            let mut n_type: u32 = 0;
            let mut pos = f.seek(SeekFrom::Current(0))?;

            while n_type != NT_GNU_BUILD_ID && n_type != NT_GO_BUILD_ID && pos <= p_end {
                let mut n_namesz = f.read_u32::<LittleEndian>()?;
                let mut n_descsz = f.read_u32::<LittleEndian>()?;
                n_type = f.read_u32::<LittleEndian>()?;

                if (n_namesz % 4) != 0 {
                    n_namesz = ((n_namesz / 4) + 1) * 4;
                }

                if (n_descsz % 4) != 0 {
                    n_descsz = ((n_descsz / 4) + 1) * 4;
                }

                let mut name = vec![0u8; n_namesz as usize];
                f.read(name.as_mut_slice())?;

                let mut desc = vec![0u8; n_descsz as usize];
                f.read(desc.as_mut_slice())?;

                let strs: Vec<String> = desc.iter().map(|b| format!("{:02x}", b)).collect();
                res = strs.join("");
                pos = f.seek(SeekFrom::Current(0))?;
            }
            if n_type != 0 {
                return Ok(res);
            }
        }
    }
    return Err(Error::new(ErrorKind::Other, "Program header PT_NOTE with NT_GNU_BUILD_ID was not found."));
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    println!("{}", get_buildid(opt.file_name)?);
    Ok(())
}
