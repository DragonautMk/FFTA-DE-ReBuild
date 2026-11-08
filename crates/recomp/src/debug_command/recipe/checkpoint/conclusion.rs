//! Explicitly concluded recipe evidence and bounded reopening provenance.
use super::*;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Evidence {run:String,checkpoint_sha256:String}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reopening {reason:String,evidence:Vec<Evidence>}
impl Reopening {
    pub(super) fn validate(&self,current:&str)->Result<(),String>{
        if self.reason.trim().is_empty() || self.reason.len()>512 || self.reason.chars().any(char::is_control) || self.evidence.is_empty() || self.evidence.len()>MAX_RUNS
            || self.evidence.iter().any(|e|!valid_id(&e.run)||e.run==current||!super::super::hash_valid(&e.checkpoint_sha256))
            || self.evidence.windows(2).any(|w|w[0].run>=w[1].run) {return Err("Invalid recipe reopening provenance".into());}Ok(())
    }
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {schema:u32,kind:String,run:String,recipe_identity:String,checkpoint_sha256:String}
fn terminal(checkpoint:&Checkpoint)->bool{
    checkpoint.steps.iter().all(|s|s.state=="completed") || checkpoint.steps.iter().any(|s|matches!(s.state.as_str(),"failed"|"assertion-failed"|"wait-timeout"))
}

// A renamed copy of the same commands is the same check. Different command
// sequences are not inferred to be equivalent merely from their outputs.
fn identity(recipe:&Recipe)->Result<String,String>{
    let bytes=serde_json::to_vec(&("ffta-recipe-evidence-v1",&recipe.rom_sha256,&recipe.steps)).map_err(|e|e.to_string())?;
    Ok(digest(&bytes))
}
fn evidence(root:&Path,base:&Path,recipe:&Recipe)->Result<Vec<Evidence>,String>{
    let current_identity=identity(recipe)?;let mut found=Vec::new();let mut count=0;
    for entry in fs::read_dir(base).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name().to_string_lossy().into_owned();if name==".run.lock"{continue;}count+=1;if count>MAX_RUNS{return Err("Recipe run directory is full or contains unexpected entries".into());}
        if !valid_id(&name){return Err("Unexpected entry in recipe run directory".into());}
        let dir=entry.path();if !crate::debug_session::regular(&dir)?.is_dir(){return Err("Recipe run must be an ordinary directory".into());}
        let path=dir.join("concluded.json");match fs::symlink_metadata(&path){Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,Err(e)=>return Err(e.to_string()),Ok(_)=>{}}
        if !crate::debug_session::regular(&path)?.is_file(){return Err("Recipe conclusion must be a regular file".into());}
        let marker:Marker=serde_json::from_slice(&crate::diagnostics::read(&path,4096)?).map_err(|e|e.to_string())?;
        if marker.schema!=1 || marker.kind!="ffta-recipe-conclusion" || marker.run!=name || !super::super::hash_valid(&marker.recipe_identity) || !super::super::hash_valid(&marker.checkpoint_sha256){return Err("Invalid concluded recipe evidence".into());}
        if marker.recipe_identity!=current_identity{continue;}
        let file=dir.join("checkpoint.json");if !crate::debug_session::regular(&file)?.is_file(){return Err("Concluded checkpoint is unavailable".into());}
        let bytes=crate::diagnostics::read(&file,MAX_CHECKPOINT)?;if digest(&bytes)!=marker.checkpoint_sha256{return Err("Concluded checkpoint changed".into());}
        let (checkpoint,_)=inspect(root,&name)?;let prior=load(root,&checkpoint.recipe_name)?;
        if !terminal(&checkpoint) || identity(&prior)?!=marker.recipe_identity{return Err("Concluded recipe evidence no longer matches its checkpoint".into());}
        found.push(Evidence{run:name,checkpoint_sha256:marker.checkpoint_sha256});
    }
    found.sort_by(|a,b|a.run.cmp(&b.run));Ok(found)
}
fn fingerprint(items:&[Evidence])->Result<String,String>{Ok(digest(&serde_json::to_vec(items).map_err(|e|e.to_string())?))}
pub(super) fn preview(root:&Path,recipe:&Recipe)->Result<(Value,String),String>{
    let runs=root.join("Data/config/debug-command-runs");
    let items=match fs::symlink_metadata(&runs){
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Vec::new(),
        Err(e)=>return Err(e.to_string()),
        Ok(_)=>{let base=base(root,false)?;let _global=runner::lock(&base,".run.lock")?;evidence(root,&base,recipe)?},
    };
    let fingerprint=fingerprint(&items)?;Ok((serde_json::to_value(items).map_err(|e|e.to_string())?,fingerprint))
}
pub(super) fn fingerprint_reopening(value:&Option<Reopening>)->Result<String,String>{fingerprint(value.as_ref().map_or(&[],|r|r.evidence.as_slice()))}
pub(super) fn authorize(root:&Path,base:&Path,recipe:&Recipe,reason:Option<&str>)->Result<Option<Reopening>,String>{
    let evidence=evidence(root,base,recipe)?;let reason=reason.unwrap_or("");
    if reason.len()>512 || reason.chars().any(char::is_control){return Err("Reopening reason must be at most 512 printable bytes".into());}
    if evidence.is_empty(){if !reason.trim().is_empty(){return Err("No matching concluded recipe evidence; remove the reopening reason".into());}return Ok(None);}
    if reason.trim().is_empty(){return Err("Matching concluded recipe evidence requires a reopening reason".into());}
    Ok(Some(Reopening{reason:reason.into(),evidence}))
}
pub(super) fn check_start(root:&Path,checkpoint:&Checkpoint,recipe:&Recipe)->Result<(),String>{
    let base=base(root,false)?;let reason=checkpoint.reopening.as_ref().map(|r|r.reason.as_str());
    if authorize(root,&base,recipe,reason)?!=checkpoint.reopening{return Err("Concluded evidence changed since preparation; prepare again".into());}Ok(())
}
fn conclude(root:&Path,id:&str)->Result<(),String>{
    let dir=runner::run_dir(root,id)?;let _execution=runner::lock(&dir,".execution.lock")?;
    let base=base(root,false)?;let _global=runner::lock(&base,".run.lock")?;
    let (checkpoint,_)=inspect(root,id)?;if !terminal(&checkpoint){return Err("Only a terminal recipe run can be marked concluded".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;let path=dir.join("checkpoint.json");let bytes=crate::diagnostics::read(&path,MAX_CHECKPOINT)?;
    let marker=Marker{schema:1,kind:"ffta-recipe-conclusion".into(),run:id.into(),recipe_identity:identity(&recipe)?,checkpoint_sha256:digest(&bytes)};
    let mut file=tempfile::NamedTempFile::new_in(&dir).map_err(|e|e.to_string())?;file.write_all(&serde_json::to_vec_pretty(&marker).map_err(|e|e.to_string())?).and_then(|_|file.as_file().sync_all()).map_err(|e|e.to_string())?;
    file.persist_noclobber(dir.join("concluded.json")).map_err(|e|e.error.to_string())?;Ok(())
}
pub(super) fn command(root:&Path,id:&str)->Value{
    match conclude(root,id){Ok(())=>json!({"schema":1,"kind":"ffta-recipe-conclusion-result","ok":true,"outcome":"concluded","run":id,"message":"Terminal checkpoint marked concluded; matching future runs require a reopening reason."}),Err(e)=>crate::debug_command::local_error("invalid-request",&e)}
}
