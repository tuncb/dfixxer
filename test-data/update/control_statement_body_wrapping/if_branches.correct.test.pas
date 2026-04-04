program IfBodyWrapFixture;

begin
  if LThis then
  begin
    Foo();
  end
  else
  begin
    EXIT;
  end;

  if LFirst then
  begin
    AlreadyWrapped;
  end
  else
  begin
    Continue;
  end;

  if LAlpha then
  begin
    Alpha;
    // between
  end
  else if LBeta then
  begin
    Beta;
  end
  else
  begin
    // else note
    Gamma;
  end;

  if LOuter then
  begin
    if LInner then
    begin
      Inner;
    end
    else
    begin
      InnerElse;
    end;
  end
  else
  begin
    OuterElse;
  end;

  if LSkip then
  begin
    Exit;
  end;

  if LKeep then
  begin
    Foo; // tail
  end;
end.
