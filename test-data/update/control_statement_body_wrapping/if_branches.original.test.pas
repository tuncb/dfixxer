program IfBodyWrapFixture;

begin
  if LThis then
    Foo()
  else
    EXIT;

  if LFirst then
  begin
    AlreadyWrapped;
  end
  else
    Continue;

  if LAlpha then
    Alpha
    // between
  else if LBeta then
    Beta
  else
    // else note
    Gamma;

  if LOuter then
    if LInner then
      Inner
    else
      InnerElse
  else
    OuterElse;

  if LSkip then
    Exit;

  if LKeep then
    Foo; // tail
end.
